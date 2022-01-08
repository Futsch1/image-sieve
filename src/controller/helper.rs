use std::rc::Rc;

use sixtyfps::Model;

/// Removes all items from a model
pub fn clear_model<T: 'static + Clone>(vec_model: Rc<sixtyfps::VecModel<T>>) {
    for _ in 0..vec_model.row_count() {
        vec_model.remove(0);
    }
}
