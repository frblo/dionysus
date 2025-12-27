use yrs::{Doc, Text, Transact};

pub fn initial_doc(room_id: &str) -> Doc {
    match room_id {
        "demo-room-1" => demo_doc(),
        _ => Doc::new(), // empty by default
    }
}

fn demo_doc() -> Doc {
  let doc = Doc::new();
    {
        let txt = doc.get_or_insert_text("codemirror");
        let mut txn = doc.transact_mut();
        txt.push(
            &mut txn,
            r#"EXT. BRICK'S PATIO - DAY

A gorgeous day.  The sun is shining.  But BRICK BRADDOCK, retired police detective, is sitting quietly, contemplating -- something.

The SCREEN DOOR slides open and DICK STEEL, his former partner and fellow retiree, emerges with two cold beers.

STEEL
Beer's ready!

BRICK
Are they cold?"#,
        );
    }
    doc
}
