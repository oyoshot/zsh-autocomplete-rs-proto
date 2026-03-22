use crate::app::App;
use crate::ui::popup::Popup;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn stable_hash_text(payload: &str) -> u64 {
    stable_hash_bytes(payload.as_bytes())
}

pub fn compute_reuse_token(prefix: &str, candidates_tsv: &str, app: &App, popup: &Popup) -> u64 {
    let signature = format!(
        "prefix={prefix}\ncandidates={:016x}\ncursor_row={}\ncursor_col={}\nterm_cols={}\nterm_rows={}\npopup_row={}\npopup_col={}\npopup_width={}\npopup_height={}\nfilter_text={}\n",
        stable_hash_text(candidates_tsv),
        app.cursor_row,
        app.cursor_col,
        app.term_cols,
        app.term_rows,
        popup.row,
        popup.col,
        popup.width,
        popup.height,
        app.filter_text,
    );
    // Avoid 0 so the shell can distinguish "has token" from "no token"
    // via a simple non-empty string check on the variable.
    stable_hash_bytes(signature.as_bytes()).max(1)
}

#[cfg(test)]
mod tests {
    use super::{compute_reuse_token, stable_hash_text};
    use crate::app::App;
    use crate::candidate::Candidate;
    use crate::ui::popup::Popup;

    fn make_app(prefix: &str, cursor_row: u16, cursor_col: u16) -> App {
        let candidates = vec![
            Candidate::parse_line("git\tcommand\tcommand"),
            Candidate::parse_line("gizmo\tcommand\tcommand"),
        ];
        App::new_with_term_size(
            candidates,
            prefix.to_string(),
            cursor_row,
            cursor_col,
            80,
            24,
        )
    }

    #[test]
    fn stable_hash_text_is_deterministic() {
        assert_eq!(stable_hash_text("git"), stable_hash_text("git"));
        assert_ne!(stable_hash_text("git"), stable_hash_text("gizmo"));
    }

    #[test]
    fn reuse_token_changes_when_popup_signature_changes() {
        let app = make_app("gi", 5, 2);
        let popup = Popup::compute(&app);
        let base = compute_reuse_token("gi", "git\tcommand\tcommand\n", &app, &popup);

        let moved_app = make_app("gi", 6, 2);
        let moved_popup = Popup::compute(&moved_app);
        let moved = compute_reuse_token("gi", "git\tcommand\tcommand\n", &moved_app, &moved_popup);

        assert_ne!(base, moved);
    }
}
