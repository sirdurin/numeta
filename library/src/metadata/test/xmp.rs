use crate::{Tag, metadata::xmp::get};

macro_rules! compare_metadata {
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

#[test]
fn nothing() {
	let data = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/"></x:xmpmeta>"#;
	compare_metadata!(data);
}

#[test]
fn no_xpacket() {
	let data = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description xmlns:xmp="http://ns.adobe.com/xap/1.0/">
            <xmp:CreateDate>2025-01-20T18:30:00+00</xmp:CreateDate>
            </rdf:Description>
        </rdf:RDF>
    </x:xmpmeta>"#;
	compare_metadata!(data, "xmp:CreateDate" => "2025-01-20T18:30:00+00");
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
	compare_metadata!(data, "xmp:CreateDate" => "2025-01-20T18:30:00+00");
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
	compare_metadata!(data, "xmp:CreateDate" => "2025-01-20T18:30:00+00", "xmp:CreatorTool" => "Photoshop");
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
	compare_metadata!(data, "xmpMM:InstanceID" => "uuid:239c0e7a-a320-4a2b-abef-6f3cbc516f23");
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
	compare_metadata!(data, "xmpMM:InstanceID" => "uuid:239c0e7a-a320-4a2b-abef-6f3cbc516f23", "xmpMM:VersionID" => "1");
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
	compare_metadata!(data, "dc:date[1]" => "2025-01-20");
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
	compare_metadata!(data, "dc:date[1]" => "2025-01-20", "dc:language[1]" => "en-US");
}
