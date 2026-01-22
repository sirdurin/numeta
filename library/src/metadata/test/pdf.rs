use super::{data, metadata};
use crate::metadata::{
	Tag,
	pdf::{delete, get},
};
use std::io::Cursor;

const REFERENCE: &[u8] = b"%PDF-1.0
%\xBB\xAD\xC0\xDE
1 0 obj
<</Type/Catalog/Pages 2 0 R>>
endobj
2 0 obj
<</Type/Pages/Count 1/Kids[3 0 R]/MediaBox[0 0 595 842]>>
endobj
3 0 obj
<</Type/Page/Parent 2 0 R>>
endobj
xref
0 4
0000000000 65535 f \n\
0000000015 00000 n \n\
0000000060 00000 n \n\
0000000133 00000 n \n\
trailer
<</Root 1 0 R/Size 4>>
startxref
176
%%EOF";

const BASIC: &[u8] = b"%PDF-1.0
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Count 1/Kids[3 0 R]/MediaBox[0 0 595 842]>>endobj
3 0 obj<</Type/Page/Parent 2 0 R>>endobj
xref
0 4
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000052 00000 n \n\
0000000123 00000 n \n\
trailer<</Root 1 0 R/Size 4>>
startxref
164
%%EOF";

const INFO: &[u8] = b"%PDF-1.0
1 0 obj<</Creator(LibreOffice 25.0)>>endobj
2 0 obj<</Type/Catalog/Pages 3 0 R>>endobj
3 0 obj<</Type/Pages/Count 1/Kids[4 0 R]/MediaBox[0 0 595 842]>>endobj
4 0 obj<</Type/Page/Parent 3 0 R>>endobj
xref
0 5
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000053 00000 n \n\
0000000096 00000 n \n\
0000000167 00000 n \n\
trailer<</Root 2 0 R/Size 5/Info 1 0 R>>
startxref
208
%%EOF";

const XMP: &[u8] = b"%PDF-1.0
1 0 obj<</Type/Catalog/Pages 2 0 R/Metadata 4 0 R>>endobj
2 0 obj<</Type/Pages/Count 1/Kids[3 0 R]/MediaBox[0 0 595 842]>>endobj
3 0 obj<</Type/Page/Parent 2 0 R>>endobj
4 0 obj<</Type/Metadata/Subtype/XML/Length 258>>stream
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<xmpmeta xmlns:x=\"adobe:ns:meta/\">
<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">
<rdf:Description rdf:about=\"\">
<pdf:Producer>LibreOffice 25.0</pdf:Producer>
</rdf:Description>
</rdf:RDF>
</xmpmeta>
endstream
endobj
xref
0 5
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000067 00000 n \n\
0000000138 00000 n \n\
0000000179 00000 n \n\
trailer<</Root 1 0 R/Size 5>>
startxref
509
%%EOF";

#[test]
fn get_basic() {
	metadata!(BASIC);
}

#[test]
fn delete_basic() {
	data!(BASIC, REFERENCE);
}

#[test]
fn get_info() {
	metadata!(INFO, "Creator" => "LibreOffice 25.0");
}

#[test]
fn delete_info() {
	data!(INFO, REFERENCE);
}

#[test]
fn get_xmp() {
	metadata!(XMP, "Producer" => "LibreOffice 25.0");
}

#[test]
fn delete_xmp() {
	data!(XMP, REFERENCE);
}
