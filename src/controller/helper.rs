use std::rc::Rc;

use slint::Model;

/// Removes all items from a model
pub fn clear_model<T: 'static + Clone>(vec_model: Rc<slint::VecModel<T>>) {
    for _ in 0..vec_model.row_count() {
        vec_model.remove(0);
    }
}
