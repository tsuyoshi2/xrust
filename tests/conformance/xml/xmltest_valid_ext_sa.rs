/*

James Clark XMLTEST cases - Standalone

    This contains cases that are valid XML documents.
    This contains case that are standalone and have references to external general entities .
*/

use std::convert::TryFrom;
use std::fs;
use xrust::parsexml;

#[test]
#[ignore]
fn validextsa001() {
    /*
        Test ID:valid-ext-sa-001
        Test URI:valid/ext-sa/001.xml
        Spec Sections:2.11
        Description:A combination of carriage return line feed in an external entity must be normalized to a single newline.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/001.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/001.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa002() {
    /*
        Test ID:valid-ext-sa-002
        Test URI:valid/ext-sa/002.xml
        Spec Sections:2.11
        Description:A carriage return (also CRLF) in an external entity must be normalized to a single newline.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/002.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/002.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa003() {
    /*
        Test ID:valid-ext-sa-003
        Test URI:valid/ext-sa/003.xml
        Spec Sections:3.1 4.1 [43] [68]
        Description:Test demonstrates that the content of an element can be empty. In this case the external entity is an empty file.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/003.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/003.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa004() {
    /*
        Test ID:valid-ext-sa-004
        Test URI:valid/ext-sa/004.xml
        Spec Sections:2.11
        Description:A carriage return (also CRLF) in an external entity must be normalized to a single newline.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/004.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/004.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa005() {
    /*
        Test ID:valid-ext-sa-005
        Test URI:valid/ext-sa/005.xml
        Spec Sections:3.2.1 4.2.2 [48] [75]
        Description:Test demonstrates the use of optional character and content particles within an element content. The test also show the use of external entity.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/005.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/005.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa006() {
    /*
        Test ID:valid-ext-sa-006
        Test URI:valid/ext-sa/006.xml
        Spec Sections:2.11 3.2.1 3.2.2 4.2.2 [48] [51] [75]
        Description:Test demonstrates the use of optional character and content particles within mixed element content. The test also shows the use of an external entity and that a carriage control line feed in an external entity must be normalized to a single newline.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/006.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/006.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa007() {
    /*
        Test ID:valid-ext-sa-007
        Test URI:valid/ext-sa/007.xml
        Spec Sections:4.2.2 4.4.3 [75]
        Description:Test demonstrates the use of external entity and how replacement text is retrieved and processed.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/007.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/007.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa008() {
    /*
        Test ID:valid-ext-sa-008
        Test URI:valid/ext-sa/008.xml
        Spec Sections:4.2.2 4.3.3. 4.4.3 [75] [80]
        Description:Test demonstrates the use of external entity and how replacement text is retrieved and processed. Also tests the use of an EncodingDecl of UTF-16.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/008.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/008.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa009() {
    /*
        Test ID:valid-ext-sa-009
        Test URI:valid/ext-sa/009.xml
        Spec Sections:2.11
        Description:A carriage return (also CRLF) in an external entity must be normalized to a single newline.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/009.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/009.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa011() {
    /*
        Test ID:valid-ext-sa-011
        Test URI:valid/ext-sa/011.xml
        Spec Sections:2.11 4.2.2 [75]
        Description:Test demonstrates the use of a public identifier with and external entity. The test also show that a carriage control line feed combination in an external entity must be normalized to a single newline.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/011.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/011.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa012() {
    /*
        Test ID:valid-ext-sa-012
        Test URI:valid/ext-sa/012.xml
        Spec Sections:4.2.1 4.2.2
        Description:Test demonstrates both internal and external entities and that processing of entity references may be required to produce the correct replacement text.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/012.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/012.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa013() {
    /*
        Test ID:valid-ext-sa-013
        Test URI:valid/ext-sa/013.xml
        Spec Sections:3.3.3
        Description:Test demonstrates that whitespace is handled by adding a single whitespace to the normalized value in the attribute list.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/013.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/013.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}

#[test]
#[ignore]
fn validextsa014() {
    /*
        Test ID:valid-ext-sa-014
        Test URI:valid/ext-sa/014.xml
        Spec Sections:4.1 4.4.3 [68]
        Description:Test demonstrates use of characters outside of normal ASCII range.
    */

    let testxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/014.xml").unwrap(),
    );
    let canonicalxml = parsexml::XMLDocument::try_from(
        fs::read_to_string("tests/conformance/xml/xmlconf/xmltest/valid/ext-sa/out/014.xml")
            .unwrap(),
    );

    assert!(testxml.is_ok());
    assert!(canonicalxml.is_ok());
    assert!(testxml.unwrap() == canonicalxml.unwrap());
}
