use crate::DomainResult;
use crate::ontology::{
    NoteFeedbackCounts, OntologyConcept, OntologyNote, OntologyNoteCreate, OntologyTripleCreate,
};
use crate::ports::BoxFuture;

#[allow(clippy::needless_pass_by_value)]
pub trait OntologyRepository: Send + Sync {
    fn upsert_concept(
        &self,
        concept: &OntologyConcept,
    ) -> BoxFuture<'_, DomainResult<OntologyConcept>>;

    fn add_broader_edge(
        &self,
        narrower_concept_id: &str,
        broader_concept_id: &str,
    ) -> BoxFuture<'_, DomainResult<()>>;

    fn create_note(&self, note: &OntologyNoteCreate) -> BoxFuture<'_, DomainResult<OntologyNote>>;

    fn write_triples(&self, triples: &[OntologyTripleCreate]) -> BoxFuture<'_, DomainResult<()>>;

    fn get_concept_by_qid(&self, qid: &str)
    -> BoxFuture<'_, DomainResult<Option<OntologyConcept>>>;

    fn list_broader_concepts(
        &self,
        concept_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<OntologyConcept>>>;

    fn note_feedback_counts(
        &self,
        note_id: &str,
    ) -> BoxFuture<'_, DomainResult<NoteFeedbackCounts>>;

    fn cleanup_expired_notes(
        &self,
        cutoff_ms: i64,
    ) -> BoxFuture<'_, DomainResult<usize>>;
}
