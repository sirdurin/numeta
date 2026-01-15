use crate::{Tag, metadata::xmp::get};

macro_rules! ok {
	($source:expr) => {
		let mut data = Vec::new();
        assert!(get($source, &mut data).is_ok());
		assert_eq!(data, Vec::new());
	};
	($source:expr, $($name:expr => $value:expr),*) => {
		let mut actual = Vec::new();
		let mut expected = Vec::new();
        $(expected.push(Tag { name: $name.to_string(), value: $value.to_string() });)*
        assert!(get($source, &mut actual).is_ok());
		assert_eq!(actual, expected);
	};
}

macro_rules! error {
	($data:expr) => {
		let mut metadata = Vec::new();
		assert!(get($data, &mut metadata).is_err());
	};
}

#[test]
fn nothing() {
	let data = br#""#;
	ok!(data);
}

#[test]
fn no_entries() {
	let data = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/"></x:xmpmeta>"#;
	ok!(data);
}

#[test]
fn no_xpacket() {
	let data = br#"
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:xmp="http://ns.adobe.com/xap/1.0/">
                <xmp:CreateDate>2025-01-20T18:30:00+00</xmp:CreateDate>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "CreateDate" => "2025-01-20T18:30:00+00");
}

#[test]
fn basic_namespace_x1() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:xmp="http://ns.adobe.com/xap/1.0/">
                <xmp:CreateDate>2025-01-20T18:30:00+00</xmp:CreateDate>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "CreateDate" => "2025-01-20T18:30:00+00");
}

#[test]
fn basic_namespace_x2() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:xmp="http://ns.adobe.com/xap/1.0/">
                <xmp:CreateDate>2025-01-20T18:30:00+00</xmp:CreateDate>
                <xmp:CreatorTool>Photoshop</xmp:CreatorTool>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "CreateDate" => "2025-01-20T18:30:00+00", "CreatorTool" => "Photoshop");
}

#[test]
fn mm_namespace_x1() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/">
                <xmpMM:InstanceID>uuid:239c0e7a-a320-4a2b-abef-6f3cbc516f23</xmpMM:InstanceID>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "InstanceID" => "uuid:239c0e7a-a320-4a2b-abef-6f3cbc516f23");
}

#[test]
fn mm_namespace_x2() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/">
                <xmpMM:InstanceID>uuid:239c0e7a-a320-4a2b-abef-6f3cbc516f23</xmpMM:InstanceID>
                <xmpMM:VersionID>1</xmpMM:VersionID>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "InstanceID" => "uuid:239c0e7a-a320-4a2b-abef-6f3cbc516f23", "VersionID" => "1");
}

#[test]
fn dc_namespace_x1() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
                <dc:date>2025-01-20</dc:date>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "date" => "2025-01-20");
}

#[test]
fn dc_namespace_x2() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
                <dc:date>2025-01-20</dc:date>
                <dc:language>en-US</dc:language>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data, "date" => "2025-01-20", "language" => "en-US");
}

#[test]
fn history() {
	let data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
        <rdf:RDF
            xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
            xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/"
        >
            <xmpMM:History>
                <rdf:Seq>
                    <rdf:li stEvt:action="saved" stEvt:when="2025-01-20T18:30:00"/>
                </rdf:Seq>
            </xmpMM:History>
        </rdf:RDF>
    </x:xmpmeta>"#;
	ok!(data);
}

#[test]
fn error_1() {
	let data = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/">"#;
	error!(data);
}

#[test]
fn error_2() {
	let data = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#;
	error!(data);
}
