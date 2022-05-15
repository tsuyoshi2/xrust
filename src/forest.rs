//! # xrust::forest
//!
//! A forest is a collection of [Tree]s. A [Tree] is a collection of [Node]s. A [Node] is an index into the [Tree].
//!
//! Both [Forest]s and [Tree]s use an arena allocator, so the object itself is simply an index that may be copied and cloned. However, in order to dererence the [Tree] or [Node] the [Forest] must be passed as an argument. This also makes deallocating memory difficult; the objects will persist until the entire [Forest] is freed.

use std::collections::HashMap;
use std::collections::hash_map::Iter;
use generational_arena::{Arena, Index};
use crate::qname::QualifiedName;
use crate::output::OutputDefinition;
use crate::xdmerror::{Error, ErrorKind};
use crate::value::Value;
use crate::parsexml::*;

/// A Forest. Forests contain [Tree]s. Each [Tree] is identified by a copyable value, similar to a Node value, that can be easily stored and passed as a parameter.
#[derive(Clone)]
pub struct Forest {
    a: Vec<Tree>,
}

pub type TreeIndex = usize;

impl Forest {
    pub fn new() -> Forest {
	Forest{a: vec![]}
    }
    pub fn plant_tree(&mut self) -> TreeIndex {
	let i = self.a.len();
	self.a.push(Tree::new(i));
	i
    }
    pub fn get_ref(&self, i: TreeIndex) -> Option<&Tree> {
	self.a.get(i)
    }
    pub fn get_ref_mut(&mut self, i: TreeIndex) -> Option<&mut Tree> {
	self.a.get_mut(i)
    }

    pub fn grow_tree(&mut self, s: &str) -> Result<TreeIndex, Error> {
	let d = parse(s)?;
	if d.content.len() == 0 {
	    Result::Err(Error::new(ErrorKind::Unknown, String::from("unable to parse XML")))
	} else {
	    let mut ns: HashMap<String, String> = HashMap::new();
	    let ti = self.plant_tree();
	    for c in d.content {
		let e = make_node(c, self, ti, &mut ns)?;
		self.get_ref_mut(ti).unwrap().push_doc_node(e)?;
	    }
	    Ok(ti)
	}
    }
}

/// A Tree, using an Arena Allocator.
/// Nodes can be detached, but not deleted
#[derive(Clone)]
pub struct Tree {
    i: TreeIndex,	// The index in the Forest
    a: Arena<NodeContent>,
    d: Index,	// The document node
}

impl Tree
{
    pub fn new(i: TreeIndex) -> Self {
        let mut a = Arena::new();
	let d = a.insert(
	    NodeBuilder::new(NodeType::Document).build()
	);
	Tree {
            i: i,
	    a: a,
	    d: d,
        }
    }

    fn get(&self, i: Index) -> Option<&NodeContent> {
	self.a.get(i)
    }
    fn get_mut(&mut self, i: Index) -> Option<&mut NodeContent> {
	self.a.get_mut(i)
    }
    pub fn get_doc_node(&self) -> Node {
	Node::new(self.d, self.i)
    }
    pub fn push_doc_node(&mut self, n: Node) -> Result<(), Error> {
	// Set the parent to the document node
	self.get_mut(n.0).unwrap().parent = Some(Node::new(self.d, self.i));
	// Push the node onto the doc node's children
	self.get_mut(self.d)
	    .map_or_else(
		|| Result::Err(Error::new(ErrorKind::Unknown, String::from("no document node"))),
		|e| {
		    e.children.push(n);
		    Ok(())
		}
	    )
    }

    pub fn new_element(&mut self, name: QualifiedName) -> Result<Node, Error> {
	Ok(
	    Node::new(
		self.a
		    .insert(NodeBuilder::new(NodeType::Element).name(name).build()),
		self.i
	    )
	)
    }
    pub fn new_text(&mut self, c: Value) -> Result<Node, Error> {
	Ok(
	    Node::new(
		self.a
		    .insert(NodeBuilder::new(NodeType::Text).value(c).build()),
		self.i
	    )
	)
    }
    pub fn new_attribute(&mut self, name: QualifiedName, v: Value) -> Result<Node, Error> {
	Ok(
	    Node::new(
		self.a
		    .insert(
			NodeBuilder::new(NodeType::Attribute)
			    .name(name)
			    .value(v)
			    .build()
		    ),
	    self.i
	    )
	)
    }
    pub fn new_comment(&mut self, v: Value) -> Result<Node, Error> {
        Ok(
	    Node::new(
		self.a
		    .insert(NodeBuilder::new(NodeType::Comment).value(v).build()),
		self.i
	    ),
	)
    }
    pub fn new_processing_instruction(&mut self, name: QualifiedName, v: Value) -> Result<Node, Error> {
        Ok(
	    Node::new(self.a
		       .insert(
			   NodeBuilder::new(NodeType::ProcessingInstruction)
			       .name(name)
			       .value(v)
			       .build()
		       ),
		      self.i
	    ),
	)
    }
}

fn make_node(
    n: XMLNode,
    f: &mut Forest,
    ti: TreeIndex,
    ns: &mut HashMap<String, String>,
) -> Result<Node, Error> {
    match n {
	XMLNode::Element(m, a, c) => {
	    a.iter()
		.filter(|b| {
		    match b {
			XMLNode::Attribute(qn, _) => {
			    match qn.get_prefix() {
				Some(p) => {
				    p == "xmlns"
				}
				_ => false,
			    }
			}
			_ => false,
		    }
		})
		.for_each(|b| {
		    if let XMLNode::Attribute(qn, v) = b {
			// add map from prefix to uri in hashmap
			ns.insert(qn.get_localname(), v.to_string()).map(|_| {});
		    }
		});
	    // Add element to the tree
	    let newns = match m.get_prefix() {
		Some(p) => {
		    match ns.get(&p) {
			Some(q) => Some(q.clone()),
			None => {
			    return Result::Err(Error::new(ErrorKind::Unknown, String::from("namespace URI not found for prefix")))
			}
		    }
		}
		None => None,
	    };
	    let new = f.get_ref_mut(ti).unwrap().new_element(
		QualifiedName::new(
		    newns,
		    m.get_prefix(),
		    m.get_localname(),
		)
	    )?;

	    // Attributes
	    a.iter()
		.for_each(|b| {
		    if let XMLNode::Attribute(qn, v) = b {
			match qn.get_prefix() {
			    Some(p) => {
				if p != "xmlns" {
				    let ans = ns.get(&p).unwrap_or(&"".to_string()).clone();
				    match f.get_ref_mut(ti).unwrap().new_attribute(
					QualifiedName::new(Some(ans), Some(p), qn.get_localname()),
					v.clone()
				    ) {
        				Ok(c) => {
					    new.add_attribute(f, c).expect("unable to add attribute"); // TODO: Don't Panic
					}
					Err(_) => {
					    //return Result::Err(e);
					}
      				    };
				}
				// otherwise it is a namespace declaration, see above
			    }
			    _ => {
				// Unqualified name
				match f.get_ref_mut(ti).unwrap().new_attribute(qn.clone(), v.clone()) {
        			    Ok(c) => {
					new.add_attribute(f, c).expect("unable to add attribute"); // TODO: Don't Panic
				    }
				    Err(_) => {
					//return Result::Err(e);
				    }
				}
			    }
			}
		    }
		}
		);

	    // Element content
	    for h in c.iter().cloned() {
		let g = make_node(h, f, ti, ns)?;
		new.append_child(f, g)?
	    }

	    Ok(new)
	}
	XMLNode::Attribute(_qn, _v) => {
	    // Handled in element arm
	    Result::Err(Error::new(ErrorKind::NotImplemented, String::from("not implemented")))
	}
	XMLNode::Text(v) => {
	    Ok(f.get_ref_mut(ti).unwrap().new_text(v)?)
	}
	XMLNode::Comment(v) => {
	    Ok(f.get_ref_mut(ti).unwrap().new_comment(v)?)
	}
	XMLNode::PI(m, v) => {
	    Ok(f.get_ref_mut(ti).unwrap().new_processing_instruction(QualifiedName::new(None, None, m), v)?)
	}
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NodeType {
  Document,
  Element,
  Text,
  Attribute,
  Comment,
  ProcessingInstruction,
  Unknown,
}

impl NodeType {
  pub fn to_string(&self) -> &'static str {
    match self {
      NodeType::Document => "Document",
      NodeType::Element => "Element",
      NodeType::Attribute => "Attribute",
      NodeType::Text => "Text",
      NodeType::ProcessingInstruction => "Processing-Instruction",
      NodeType::Comment => "Comment",
      NodeType::Unknown => "--None--",
    }
  }
}

impl Default for NodeType {
  fn default() -> Self {
    NodeType::Unknown
  }
}

/// Node
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Node(Index, TreeIndex);

impl Node {
    fn new(i: Index, t: TreeIndex) -> Self {
	Node(i, t)
    }

    fn get<'a>(&self, f: &'a Forest) -> Option<&'a NodeContent> {
	f.get_ref(self.1).unwrap().get(self.0)
    }

    pub fn to_string(&self, f: &Forest) -> String {
	match f.get_ref(self.1) {
	    Some(e) => e,
	    None => return String::from(""),
	};
	match self.node_type(f) {
	    NodeType::Element => {
		// TODO: string value of all descendant text nodes
		String::new()
	    }
	    NodeType::Text |
	    NodeType::Attribute |
	    NodeType::Comment => {
		self.get(f).unwrap().value().as_ref().map_or(
		    String::new(),
		    |v| v.to_string()
		)
	    }
	    _ => String::new(),
	}
    }
    pub fn to_xml(&self, f: &Forest) -> String {
	let d = match f.get_ref(self.1) {
	    Some(e) => e,
	    None => return String::from(""),
	};
	let nc = match d.get(self.0) {
	    Some(e) => e,
	    None => return String::from(""),
	};
	match nc.node_type() {
	    NodeType::Element => {
		let mut result = String::from("<");
		let name = nc.name().as_ref().unwrap();
		result.push_str(name.to_string().as_str());
		nc.attributes.iter().for_each(|(k, v)| {
		    result.push(' ');
		    result.push_str(k.to_string().as_str());
		    result.push_str("='");
		    result.push_str(v.to_string(f).as_str());
		    result.push('\'');
		});
		result.push_str(">");
		let mut children = self.child_iter();
		loop {
		    match children.next(f) {
			Some(c) => result.push_str(c.to_xml(f).as_str()),
			None => break,
		    }
		};
		result.push_str("</");
		result.push_str(name.to_string().as_str());
		result.push_str(">");
		result
	    }
	    NodeType::Text => {
		nc.value().as_ref().unwrap().to_string()
	    }
	    NodeType::Comment => {
		let mut result = String::from("<!--");
		result.push_str(nc.value().as_ref().unwrap().to_string().as_str());
		result.push_str("-->");
		result
	    }
	    NodeType::ProcessingInstruction => {
		let mut result = String::from("<?");
		result.push_str(nc.name().as_ref().unwrap().to_string().as_str());
		result.push(' ');
		result.push_str(nc.value().as_ref().unwrap().to_string().as_str());
		result.push_str("?>");
		result
	    }
	    _ => {
		// TODO
		String::from("-- not implemented --")
	    }
	}
    }
    pub fn to_xml_with_options(&self, _f: &Forest, _od: &OutputDefinition) -> String {
	String::from("not implemented yet")
    }
    pub fn to_json(&self, _f: &Forest) -> String {
	String::from("not implemented yet")
    }

    pub fn to_int(&self, f: &Forest) -> Result<i64, Error> {
	// Convert to a string, then try parsing that as an integer
	self.to_string(f).parse::<i64>()
	    .map_err(|e| Error::new(ErrorKind::Unknown, e.to_string()))
    }
    pub fn to_double(&self, f: &Forest) -> f64 {
	// Convert to a string, then try parsing that as a double
	match self.to_string(f).parse::<f64>() {
	    Ok(g) => g,
	    Err(_) => f64::NAN,
	}
    }
    pub fn to_name(&self, f: &Forest) -> QualifiedName {
	f.get_ref(self.1)
	    .map_or(
		QualifiedName::new(None, None, String::from("")),
		|d| d.get(self.0)
		    .map_or(
			QualifiedName::new(None, None, String::from("")),
			|o| o.name().as_ref().map_or(
			    QualifiedName::new(None, None, String::from("")),
			    |p| p.clone(),
			)
		    ),
	    )
    }

    pub fn node_type(&self, f: &Forest) -> NodeType {
	f.get_ref(self.1)
	    .map_or(
		NodeType::Unknown,
		|d| d.get(self.0)
		    .map_or(
			NodeType::Unknown,
			|m| m.node_type(),
		    ),
	    )
    }

    pub fn append_child(&self, f: &mut Forest, c: Node) -> Result<(), Error> {
	// Check that self is an element and that c is not an attribute
        if self.node_type(f) != NodeType::Element {
            return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("must be an element"),
            ));
        }
        if c.node_type(f) == NodeType::Attribute {
            return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("cannot append an attribute as a child"),
            ));
        }

	let d = match f.get_ref_mut(self.1) {
	    Some(e) => e,
	    None=> return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("unable to find tree"),
            )),
	};

	// TODO: If c is in a different Tree, then deep copy

	// TODO: detach c from wherever it is currently located

	// self will now be c's parent
	match d.get_mut(c.0) {
	    Some(e) => {
		e.parent = Some(self.clone())
	    }
	    None => return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("unable to find node"),
            ))
	}

	// Push c onto self's child list
	match d.get_mut(self.0) {
	    Some(e) => e.children.push(c),
	    None => return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("unable to find node"),
            ))
	}

        Ok(())
    }
    pub fn insert_before(&mut self, _f: &mut Forest, _insert: Node) -> Result<(), Error> {
        return Result::Err(Error::new(
            ErrorKind::NotImplemented,
            String::from("not yet implemented"),
        ));
    }

    /// Detach the node from the tree
    pub fn remove(&self, f: &mut Forest) -> Result<(), Error> {
	let d = match f.get_ref_mut(self.1) {
	    Some(e) => e,
	    None => return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("unable to find tree"),
            ))
	};

	// Remove from parent's child list
	let cl = &mut d.get_mut(d.get(self.0).unwrap().parent.unwrap().0).unwrap().children;
	let i = cl.iter()
	    .enumerate()
	    .skip_while(|(_, x)| x.0 != self.0)
	    .nth(0)
	    .map(|(e, _)| e)
	    .unwrap();
	cl.remove(i);

	// This node now has no parent
	d.get_mut(self.0).unwrap().parent = None;

	Ok(())
    }

    pub fn add_attribute(&self, f: &mut Forest, a: Node) -> Result<(), Error> {
        if self.node_type(f) != NodeType::Element {
            return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("must be an element"),
            ));
        }
        if a.node_type(f) != NodeType::Attribute {
            return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("argument must be an attribute"),
            ));
        }

	let d = match f.get_ref_mut(self.1) {
	    Some(e) => e,
	    None => return Result::Err(Error::new(
                ErrorKind::Unknown,
                String::from("unable to find tree"),
            ))
	};

	// TODO: detach a from wherever it is currently located

	// self will now be a's parent
	d.get_mut(a.0).unwrap().parent = Some(self.clone());
	// Add a to self's attribute hashmap
	let qn = d.get(a.0).unwrap().name().as_ref().unwrap().clone();
	d.get_mut(self.0).unwrap().attributes.insert(qn, a);
	Ok(())
    }

    pub fn ancestor_iter(&self) -> Ancestors {
	Ancestors::new(self.0, self.1)
    }
    pub fn parent(&self, f: &Forest) -> Option<Node> {
	self.ancestor_iter().next(f).map(|p| p)
    }
    pub fn child_iter(&self) -> Children {
	Children::new(self.0, self.1)
    }
    pub fn next_iter(&self, f: &Forest) -> Siblings {
	Siblings::new(self.0, self.1, 1, f)
    }
    pub fn prev_iter(&self, f: &Forest) -> Siblings {
	Siblings::new(self.0, self.1, -1, f)
    }
    pub fn descend_iter(&self, f: &Forest) -> Descendants {
	Descendants::new(self.0, self.1, f)
    }

    pub fn attribute_iter<'a>(&self, f: &'a Forest) -> Attributes<'a> {
	Attributes::new(self.0, f.get_ref(self.1).unwrap())
    }
    pub fn get_attribute(&self, f: &Forest, qn: &QualifiedName) -> Option<Node> {
	match f.get_ref(self.1) {
	    Some(d) => {
		match d.get(self.0) {
		    Some(nc) => {
			match nc.attributes.get(qn) {
			    Some(m) => Some(m.clone()),
			    None => None,
			}
		    }
		    None => None,
		}
	    }
	    None => None,
	}
    }

    /// Convenience method that returns if this is an element type node
    pub fn is_element(&self, f: &Forest) -> bool {
	match f.get_ref(self.1) {
	    Some(d) => {
		match d.get(self.0) {
		    Some(nc) => {
			nc.t == NodeType::Element
		    }
		    None => false,
		}
	    }
	    None => false,
	}
    }
}

pub struct Ancestors {
    t: TreeIndex,
    cur: Index,
}

impl Ancestors {
    fn new(cur: Index, t: TreeIndex) -> Ancestors {
	Ancestors{t, cur}
    }
    pub fn next(&mut self, f: &Forest) -> Option<Node> {
	if let Some(d) = f.get_ref(self.t) {
	    if let Some(c) = d.get(self.cur) {
		if let Some(p) = c.parent {
		    if p.node_type(f) == NodeType::Document {
			None
		    } else {
			self.cur = p.0;
			Some(p)
		    }
		} else {
		    None
		}
	    } else {
		None
	    }
	} else {
	    None
	}
    }
}

pub struct Descendants {
    t: TreeIndex,
    start: Index,
    cur: Index,
    stack: Vec<(Index, usize)>,
}

impl Descendants {
    fn new(cur: Index, t: TreeIndex, f: &Forest) -> Descendants {
	// Find cur in the parent's child list
	let d = f.get_ref(t).unwrap();
	let pi = d.get(cur).unwrap().parent.unwrap().0;
	let p = d.get(pi).unwrap();
	let q = p.children.iter().enumerate()
	    .skip_while(|(_, i)| i.0 != cur)
	    .nth(0)
	    .map(|(e, _)| e)
	    .unwrap();
	Descendants{
	    t,
	    start: cur,
	    cur: cur,
	    stack: vec![(pi, q)],
	}
    }
    pub fn next(&mut self, f: &Forest) -> Option<Node> {
	if self.stack.is_empty() {
	    None
	} else {
	    // Return the first child,
	    // otherwise return the next sibling
	    // otherwise return an ancestor's next sibling
	    // (don't go past start)
	    match Node::new(self.cur, self.t).child_iter().next(f) {
		Some(n) => {
		    self.stack.push((self.cur, 0));
		    self.cur = n.0;
		    Some(n)
		}
		None => {
		    let d = f.get_ref(self.t).unwrap();
		    let (i, mut s) = self.stack.last_mut().unwrap();
		    let pnc = d.get(*i).unwrap();
		    if pnc.children.len() < s {
			// have a next sibling
			s += 1;
			self.cur = pnc.children.get(s).unwrap().0;
			Some(Node::new(self.cur, self.t))
		    } else {
			// ancestor next sibling
			let result: Option<Node>;
			loop {
			    self.stack.pop();
			    if self.stack.is_empty() {
				result = None;
				break
			    } else {
				let l = self.stack.last_mut().unwrap();
				let (j, mut t) = l;
				let qnc = d.get(*j).unwrap();
				if qnc.children.len() > t + 1 {
				    t += 1;
				    *l = (*j, t);
				    self.cur = qnc.children.get(t).unwrap().0;
				    result = Some(Node::new(self.cur, self.t));
				    break
				} else {
				    if *j == self.start {
					result = None;
					break
				    }
				}
			    }
			}
			result
		    }
		}
	    }
	}
    }
}

pub struct Children {
    t: TreeIndex,
    parent: Index,
    cur: usize,
}

impl Children {
    fn new(parent: Index, t: TreeIndex) -> Children {
	Children{t, parent, cur: 0}
    }
    pub fn next(&mut self, f: &Forest) -> Option<Node> {
	if let Some(d) = f.get_ref(self.t) {
	    if let Some(n) = d.get(self.parent) {
		if n.children.len() > self.cur {
		    self.cur += 1;
		    Some(n.children[self.cur - 1])
		} else {
		    None
		}
	    } else {
		None
	    }
	} else {
	    None
	}
    }
}

pub struct Siblings {
    t: TreeIndex,
    parent: Index,
    cur: usize,
    dir: i16,
}

impl Siblings {
    fn new(n: Index, t: TreeIndex, dir: i16, f: &Forest) -> Siblings {
	let d = f.get_ref(t).unwrap();
	let nc = d.get(n).unwrap();
	let pnc = d.get(nc.parent.unwrap().0).unwrap();
	let cur = pnc.children.iter().enumerate()
	    .skip_while(|(_, i)| i.0 != n)
	    .nth(0)
	    .map(|(e, _)| e)
	    .unwrap();
	Siblings{
	    t,
	    parent: nc.parent.unwrap().0,
	    dir,
	    cur: cur,
	}
    }
    pub fn next(&mut self, f: &Forest) -> Option<Node> {
	if let Some(d) = f.get_ref(self.t) {
	    if let Some(n) = d.get(self.parent) {
		if self.dir > 0 && n.children.len() > self.cur + 1 {
		    self.cur += 1;
		    Some(n.children[self.cur])
		} else if self.dir < 0 && self.cur > 0 {
		    self.cur -= 1;
		    Some(n.children[self.cur])
		} else {
		    None
		}
	    } else {
		None
	    }
	} else {
	    None
	}
    }
}

pub struct Attributes<'a>{
    it: Iter<'a, QualifiedName, Node>,
}

impl<'a> Attributes<'a> {
    fn new(i: Index, d: &'a Tree) -> Attributes {
	Attributes{
	    it: d.get(i).unwrap().attributes.iter()
	}
    }
    pub fn next(&mut self) -> Option<Node> {
	self.it.next().map(|(_, n)| *n)
    }
}

#[derive(Clone, Default)]
pub struct NodeContent {
    t: NodeType,
    name: Option<QualifiedName>,
    v: Option<Value>,
    parent: Option<Node>, // The document node has no parent
    attributes: HashMap<QualifiedName, Node>, // for non-elements nodes this is always. Should this be an Option?
    children: Vec<Node>, // for non-element nodes this is always empty. Should this be an Option?
}

impl NodeContent {
    pub fn new(t: NodeType) -> Self {
        NodeContent {
	    t,
            ..Default::default()
        }
    }
    pub fn node_type(&self) -> NodeType {
	self.t
    }
    pub fn name(&self) -> &Option<QualifiedName> {
        &self.name
    }
    pub fn value(&self) -> &Option<Value> {
	&self.v
    }
}

struct NodeBuilder(NodeContent);

impl NodeBuilder {
    pub fn new(t: NodeType) -> Self {
        NodeBuilder(NodeContent::new(t))
    }
    pub fn name(mut self, qn: QualifiedName) -> Self {
        self.0.name = Some(qn);
        self
    }
    // Q: what to do if the node already has a value?
    // This implementation drops the previous value
    pub fn value(mut self, v: Value) -> Self {
        self.0.v = Some(v);
        self
    }
    pub fn build(self) -> NodeContent {
        self.0
    }
}

/// Nodes
///
/// A document contains [Node] objects.
pub trait NodeTrait {
    /// Return the string value of the [Node]
    fn to_string(&self) -> String;
    /// Serialize the given [Node] as XML
    fn to_xml(&self) -> String;
    /// Serialize as XML, with options
    fn to_xml_with_options(&self, od: &OutputDefinition) -> String;
    /// Serialize as JSON
    fn to_json(&self) -> String;
    /// Determine the effective boolean value. See XPath 2.4.3.
    /// A Document or Node always returns true.
    fn to_bool(&self) -> bool {
	true
    }
    /// Return the integer value. For a Document, this is a type error.
    fn to_int(&self) -> Result<i64, Error>;
    /// Return the double value. For a Document, this is a type error, i.e. NaN.
    fn to_double(&self) -> f64;
    /// Gives the name of the [Node]. Documents do not have a name, so the implementation must return an empty string.
    fn to_name(&self) -> QualifiedName;

    /// Return the type of a Node
    fn node_type(&self) -> NodeType;

    /// Callback for logging/debugging, particularly in a web_sys environment
    fn log(&self, _m: &str) {
	// Noop
    }

    /// Return the root node of the Document.
    //fn get_root_element(&self) -> Option<N>;
    /// Set the root element for the Document. If the Document already has a root element then it will be removed. The node must be an element. If the node supplied is of a different concrete type to the Document then an error is returned. If the element is from a different Document, then the function performs a deep copy.
    //fn set_root_element(&mut self, r: Self::Node) -> Result<(), Error>;

    /// An iterator over ancestors of a [Node].
    //fn ancestor_iter<D: Document<N>>(&self, n: N) -> Box<dyn AncestorIterator<D, N, Item = N>>;
    /// Navigate to the parent of a [Node]. Documents, and the root element, don't have a parent, so the default implementation returns None. This is a convenience function for ancestor_iter.
    fn parent(&self) -> Option<Node> {
	None
    }
    /// An iterator for the child nodes of a [Node]. Non-element type nodes will immediately return None.
    //fn child_iter<D: Document<N>>(&self, n: N) -> Box<dyn ChildIterator<D, N, Item = N>>;
    /// An iterator for the child nodes of the Document. This may include the prologue, root element, and epilogue.
    //fn doc_child_iter<D: Document<N>>(&self) -> Box<dyn DocChildIterator<D, N, Item = N>>;
    /// An iterator for descendants of a [Node]. Does not include the [Node] itself.
    // fn descend_iter(&self, n: Box<dyn Node>) -> Box<dyn Iterator<Item = Box<dyn Node>>>;
    /// An iterator for following siblings of a [Node]. Does not include the [Node] itself.
    // fn following_sibling_iter(&self, n: Box<dyn Node>) -> Box<dyn Iterator<Item = Box<dyn Node>>>;
    /// An iterator for preceding siblings of a [Node]. Does not include the [Node] itself.
    // fn preceding_sibling_iter(&self, n: Box<dyn Node>) -> Box<dyn Iterator<Item = Box<dyn Node>>>;

    /// Create an element [Node] in the Document.
    fn new_element(&mut self, name: QualifiedName) -> Result<Node, Error>;
    /// Create a text [Node] in the Document.
    fn new_text(&mut self, c: Value) -> Result<Node, Error>;
    /// Create an attribute [Node] in the Document.
    fn new_attribute(&mut self, name: QualifiedName, v: Value) -> Result<Node, Error>;
    /// Create a comment [Node] in the Document.
    fn new_comment(&mut self, v: Value) -> Result<Node, Error>;
    /// Create a processing instruction [Node] in the Document.
    fn new_processing_instruction(&mut self, name: QualifiedName, v: Value) -> Result<Node, Error>;

    /// Append a [Node] to the children of a [Node]. If the [Node] to be appended is from a different Document then this function performs a deep copy.
    fn append_child(&mut self, parent: Node, child: Node) -> Result<(), Error>;
    /// Inserts a [Node] (insert) before another [Node] (child) in the children of it's parent element [Node]. If the [Node] to be inserted is from a different Document then this function performs a deep copy.
    fn insert_before(&mut self, child: Node, insert: Node) -> Result<(), Error>;
    // TODO: replace_child

    /// Add an attribute [Node] to an element type [Node]. If the attribute [Node] is from a different Document then this function adds a copy of the attribute [Node].
    fn add_attribute_node(&mut self, _parent: Node, _a: Node) -> Result<(), Error> {
	Result::Err(Error::new(ErrorKind::NotImplemented, String::from("not implemented")))
    }

    /// Remove a node from its parent
    fn remove(&mut self, _n: Node) -> Result<(), Error> {
	Result::Err(Error::new(ErrorKind::NotImplemented, String::from("not implemented")))
    }
}

/// An iterator over ancestor nodes
pub trait AncestorIterator {
    type Node;
    fn next(&mut self, t: Tree) -> Option<Self::Node>;
}

/// An iterator over child nodes
pub trait ChildIterator {
    type Node;
    fn next(&mut self, t: Tree) -> Option<Self::Node>;
}

/// An iterator over child nodes of a [Document]
pub trait DocChildIterator {
    type Node;
    fn next(&mut self, t: Tree) -> Option<Self::Node>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn emptydoc() {
	let mut f = Forest::new();
	f.plant_tree();
	assert!(true)
    }

    #[test]
    fn root_element() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	assert_eq!(e.to_xml(&f), "<Test></Test>")
    }

    #[test]
    fn add_element() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	assert_eq!(e.to_xml(&f), "<Test><Level-1></Level-1></Test>")
    }

    #[test]
    fn add_text() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let txt = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("this is a test"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, txt).expect("unable to append node");
	assert_eq!(e.to_xml(&f), "<Test><Level-1>this is a test</Level-1></Test>")
    }

    #[test]
    fn add_attribute() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let txt = f.get_ref_mut(ti).unwrap()
	    .new_attribute(QualifiedName::new(None, None, String::from("data")), Value::from("this is a test"))
	    .expect("unable to create text node");
	l1.add_attribute(&mut f, txt).expect("unable to add attribute");
	assert_eq!(e.to_xml(&f), "<Test><Level-1 data='this is a test'></Level-1></Test>")
    }

    #[test]
    fn add_comment() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let c = f.get_ref_mut(ti).unwrap()
	    .new_comment(Value::from("this is a comment"))
	    .expect("unable to create comment node");
	l1.append_child(&mut f, c).expect("unable to append node");
	assert_eq!(e.to_xml(&f), "<Test><Level-1><!--this is a comment--></Level-1></Test>")
    }

    #[test]
    fn add_pi() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap().push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let pi = f.get_ref_mut(ti).unwrap()
	    .new_processing_instruction(QualifiedName::new(None, None, String::from("testPI")), Value::from("this is a PI"))
	    .expect("unable to create processing instruction node");
	l1.append_child(&mut f, pi).expect("unable to append node");
	assert_eq!(e.to_xml(&f), "<Test><Level-1><?testPI this is a PI?></Level-1></Test>")
    }

    #[test]
    fn remove() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap().push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");
	l1.remove(&mut f).expect("unable to remove node");
	assert_eq!(e.to_xml(&f), "<Test><Level-1>two</Level-1></Test>");
    }

    #[test]
    fn children() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap().push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");

	let mut children = e.child_iter();
	assert_eq!(children.next(&f), Some(l1));
	assert_eq!(children.next(&f), Some(l2));
	assert_eq!(children.next(&f), None)
    }

    #[test]
    fn ancestors() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap().push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");

	let mut ancestors = t2.ancestor_iter();
	assert_eq!(ancestors.next(&f), Some(l2));
	assert_eq!(ancestors.next(&f), Some(e));
	assert_eq!(ancestors.next(&f), None)
    }

    #[test]
    fn parent() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");

	assert_eq!(t2.parent(&f), Some(l2));
    }

    #[test]
    fn following_sibling() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");

	let mut follow = l1.next_iter(&f);
	assert_eq!(follow.next(&f), Some(l2));
	assert_eq!(follow.next(&f), None)
    }

    #[test]
    fn preceding_sibling() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");

	let mut pre = l2.prev_iter(&f);
	assert_eq!(pre.next(&f), Some(l1));
	assert_eq!(pre.next(&f), None)
    }

    #[test]
    fn descendants() {
	let mut f = Forest::new();
	let ti = f.plant_tree();
	let e = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Test")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(e)
	    .expect("unable to add node to doc");
	let g = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Another")))
	    .expect("unable to create element node");
	f.get_ref_mut(ti).unwrap()
	    .push_doc_node(g)
	    .expect("unable to add node to doc");
	let l1 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l1).expect("unable to append node");
	let t1 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("one"))
	    .expect("unable to create text node");
	l1.append_child(&mut f, t1).expect("unable to append node");
	let l2 = f.get_ref_mut(ti).unwrap()
	    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
	    .expect("unable to create element node");
	e.append_child(&mut f, l2).expect("unable to append node");
	let t2 = f.get_ref_mut(ti).unwrap()
	    .new_text(Value::from("two"))
	    .expect("unable to create text node");
	l2.append_child(&mut f, t2).expect("unable to append node");

	assert_eq!(e.to_xml(&f), "<Test><Level-1>one</Level-1><Level-1>two</Level-1></Test>");

	let mut desc = e.descend_iter(&f);
	assert_eq!(desc.next(&f), Some(l1));
	assert_eq!(desc.next(&f), Some(t1));
	assert_eq!(desc.next(&f), Some(l2));
	assert_eq!(desc.next(&f), Some(t2));
	assert_eq!(desc.next(&f), None)
    }

    #[test]
    fn parse() {
	let mut f = Forest::new();
	let ti = f.grow_tree("<Test><empty/>
<data mode='mixed'>This contains <i>mixed</i> content.</data>
<special>Some escaped chars &lt;&amp;&gt;</special>
</Test>")
	    .expect("unable to parse");
	assert_eq!(f.get_ref(ti).unwrap().get_doc_node().child_iter().next(&f).unwrap().to_xml(&f), "<Test><empty></empty>
<data mode='mixed'>This contains <i>mixed</i> content.</data>
<special>Some escaped chars <&></special>
</Test>")
    }

    #[bench]
    fn bench_tree(b: &mut Bencher) {
	b.iter(|| {
	    let mut f = Forest::new();
	    let ti = f.plant_tree();
	    let r = f.get_ref_mut(ti).unwrap()
		.new_element(QualifiedName::new(None, None, String::from("Test")))
		.expect("unable to create element node");
	    f.get_ref_mut(ti).unwrap()
		.push_doc_node(r)
		.expect("unable to add doc node");
	    (1..3).for_each(|i| {
		let j = f.get_ref_mut(ti).unwrap()
		    .new_element(QualifiedName::new(None, None, String::from("Level-1")))
		    .expect("unable to create element node");
		r.append_child(&mut f, j).expect("unable to append node");
		(1..3).for_each(|k| {
		    let l = f.get_ref_mut(ti).unwrap()
			.new_element(QualifiedName::new(None, None, String::from("Level-2")))
			.expect("unable to create element node");
		    j.append_child(&mut f, l).expect("unable to append node");
		    let m = f.get_ref_mut(ti).unwrap()
			.new_text(Value::from(format!("node {}-{}", i, k)))
			.expect("unable to create text node");
		    l.append_child(&mut f, m).expect("unable to append node");
		});
	    });
	})
    }
}
