use crate::DomainResult;
use crate::ontology::OntologyEdgeKind;
use crate::ontology::{
    NoteFeedbackCounts, OntologyActionRef, OntologyConcept, OntologyNote, OntologyNoteCreate,
    OntologyPlaceRef, OntologyTripleCreate,
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

    fn list_note_edge_targets(
        &self,
        note_id: &str,
        edge: OntologyEdgeKind,
    ) -> BoxFuture<'_, DomainResult<Vec<String>>>;

    fn get_concept_by_qid(&self, qid: &str)
    -> BoxFuture<'_, DomainResult<Option<OntologyConcept>>>;

    fn get_concepts_by_qids(
        &self,
        qids: &[String],
    ) -> BoxFuture<'_, DomainResult<Vec<OntologyConcept>>>;

    fn get_actions_by_types(
        &self,
        action_types: &[String],
    ) -> BoxFuture<'_, DomainResult<Vec<OntologyActionRef>>>;

    fn get_places_by_ids(
        &self,
        place_ids: &[String],
    ) -> BoxFuture<'_, DomainResult<Vec<OntologyPlaceRef>>>;

    fn list_broader_concepts(
        &self,
        concept_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<OntologyConcept>>>;

    fn note_feedback_counts(
        &self,
        note_id: &str,
    ) -> BoxFuture<'_, DomainResult<NoteFeedbackCounts>>;

    fn cleanup_expired_notes(&self, cutoff_ms: i64) -> BoxFuture<'_, DomainResult<Vec<String>>>;
}
