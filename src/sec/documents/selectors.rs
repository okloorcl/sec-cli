use super::submission::SubmissionDocument;

pub struct DocumentSet<'a> {
    docs: &'a [SubmissionDocument],
}

impl<'a> DocumentSet<'a> {
    pub fn new(docs: &'a [SubmissionDocument]) -> Self {
        Self { docs }
    }

    pub fn form4_ownership_xml(&self) -> impl Iterator<Item = &'a SubmissionDocument> {
        self.docs.iter().filter(|doc| {
            doc.xml_content().contains("<ownershipDocument")
                || doc.is_type("4")
                || doc.is_type("4/A")
        })
    }

    pub fn thirteenf_information_tables(&self) -> impl Iterator<Item = &'a SubmissionDocument> {
        self.docs.iter().filter(|doc| {
            let content = doc.xml_content();
            content.contains("<informationTable")
                || content.contains(":informationTable")
                || doc.is_type("INFORMATION TABLE")
        })
    }

    pub fn thirteenf_primary_documents(&self) -> impl Iterator<Item = &'a SubmissionDocument> {
        self.docs.iter().filter(|doc| {
            let content = doc.xml_content();
            content.contains("<edgarSubmission")
                || (doc.is_primary()
                    && (doc.is_type("13F-HR")
                        || doc.is_type("13F-HR/A")
                        || doc.is_type("13F-NT")
                        || doc.is_type("13F-NT/A")))
        })
    }

    pub fn primary_documents(&self) -> impl Iterator<Item = &'a SubmissionDocument> {
        self.docs.iter().filter(|doc| {
            doc.sequence
                .as_deref()
                .map(|value| value.trim() == "1")
                .unwrap_or(false)
        })
    }

    pub fn by_type(&self, document_type: &str) -> impl Iterator<Item = &'a SubmissionDocument> {
        self.docs
            .iter()
            .filter(move |doc| doc.is_type(document_type))
    }
}
