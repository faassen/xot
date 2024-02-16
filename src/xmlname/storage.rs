// A way to store an XML name without any references.
//
// Needs a Xot and a context to be used.

use crate::id::{NameId, PrefixId};

struct XmlNameStorage {
    name_id: NameId,
    prefix_id: PrefixId,
}
