use daggy::Dag;

use crate::types::Makefile;

fn from_makefile<'a>(i: Makefile) -> Result<Dag<String, u32, u32>, &'a str> {
    let d = Dag::new();
    Ok(d)
}
