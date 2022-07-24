use lalrpop_util::lalrpop_mod;

use crate::ViewPath;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused)]
    path
);

/// Parses the string as a lookup path.
pub fn parse_view_path(s: &str) -> Result<ViewPath, String> {
    path::ViewPathParser::new()
        .parse(s)
        .map_err(|err| format!("{}", err))
}
