use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Url};
use yag_template_analysis::{AnalysisError, AnalysisWarning};
use yag_template_syntax::SyntaxError;

use crate::session::{Document, Session};

pub(crate) async fn publish(sess: &Session, uri: &Url) -> anyhow::Result<()> {
    let doc = sess.document(uri)?;

    let syntax_error_diags = doc.parse.errors.iter().map(|err| diag_for_syntax_error(&doc, err));
    let analysis_error_diags = doc.analysis.errors.iter().map(|err| diag_for_analysis_error(&doc, err));
    let analysis_warning_diags = doc
        .analysis
        .warnings
        .iter()
        .map(|warning| diag_for_analysis_warning(&doc, warning));
    let all_diags = syntax_error_diags
        .chain(analysis_error_diags)
        .chain(analysis_warning_diags)
        .collect();

    let version = Default::default();
    sess.client
        .publish_diagnostics(uri.clone(), all_diags, Some(version))
        .await;
    Ok(())
}

fn diag_for_syntax_error(doc: &Document, err: &SyntaxError) -> Diagnostic {
    Diagnostic::new_simple(doc.mapper.range(err.range), err.message.clone())
}

fn diag_for_analysis_error(doc: &Document, err: &AnalysisError) -> Diagnostic {
    Diagnostic::new_simple(doc.mapper.range(err.range), err.message.clone())
}

fn diag_for_analysis_warning(doc: &Document, warning: &AnalysisWarning) -> Diagnostic {
    Diagnostic::new(
        doc.mapper.range(warning.range),
        Some(DiagnosticSeverity::WARNING),
        None,
        None,
        warning.message.clone(),
        None,
        None,
    )
}

pub(crate) async fn clear(sess: &Session, uri: &Url) {
    let version = Default::default();
    sess.client
        .publish_diagnostics(uri.clone(), Vec::new(), Some(version))
        .await;
}
