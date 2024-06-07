use egui::Response;

pub type ResponseCallback<'a> = dyn FnMut(Response, &String) + 'a;

pub struct RenderHooks<'a> {
    pub(crate) response_callback: Option<Box<ResponseCallback<'a>>>,
}

impl<'a> RenderHooks<'a> {
    pub fn response_callback(&mut self, response: Response, pointer_str: &String) {
        if let Some(response_callback) = self.response_callback.as_mut() {
            response_callback(response, pointer_str)
        }
    }
}
