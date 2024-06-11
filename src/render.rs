use std::fmt::Display;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, FontId, Label, Response, Sense, TextFormat, Ui,
};

use crate::{
    delimiters::Punc,
    pointer::{JsonPointer, NestedProperty},
    search::SearchTerm,
    value::{BaseValueType, ToJsonTreeValue},
    JsonTreeStyle,
};

type ResponseCallback<'a> = dyn FnMut(Response, &String) + 'a;
type RenderKeyHook<'a> = dyn FnMut(&mut Ui, &RenderKeyContext<'a, '_>) -> Option<Response> + 'a;
type RenderValueHook<'a, T> =
    dyn FnMut(&mut Ui, &RenderValueContext<'a, '_, T>) -> Option<Response> + 'a;
type PostRenderValueHook<'a, T> = dyn FnMut(&mut Ui, &RenderValueContext<'a, '_, T>) + 'a;
type ResponseHook<'a> = dyn FnMut(ResponseContext<'a, '_>) + 'a;

pub struct RenderValueContext<'a, 'b, T: ToJsonTreeValue> {
    pub value: &'a T,
    pub display_value: &'a dyn Display,
    pub value_type: BaseValueType,
    pub pointer: JsonPointer<'a, 'b>,
}

pub struct RenderKeyContext<'a, 'b> {
    pub key: NestedProperty<'a>,
    pub pointer: JsonPointer<'a, 'b>,
}

pub(crate) struct RenderPuncContext<'a, 'b> {
    pub(crate) punc: Punc<'static>,
    pub(crate) pointer: JsonPointer<'a, 'b>,
}

pub struct ResponseContext<'a, 'b> {
    pub response: Response,
    pub pointer: JsonPointer<'a, 'b>,
}

pub(crate) struct RenderHooks<'a, T: ToJsonTreeValue> {
    pub(crate) response_callback: Option<Box<ResponseCallback<'a>>>,
    pub(crate) render_key_hook: Option<Box<RenderKeyHook<'a>>>,
    pub(crate) render_value_hook: Option<Box<RenderValueHook<'a, T>>>,
    pub(crate) post_render_value_hook: Option<Box<PostRenderValueHook<'a, T>>>,
    pub(crate) response_hook: Option<Box<ResponseHook<'a>>>,
}

impl<'a, T: ToJsonTreeValue> Default for RenderHooks<'a, T> {
    fn default() -> Self {
        Self {
            response_callback: None,
            render_key_hook: None,
            render_value_hook: None,
            post_render_value_hook: None,
            response_hook: None,
        }
    }
}

pub(crate) struct JsonTreeRenderer<'a, T: ToJsonTreeValue> {
    pub(crate) style: JsonTreeStyle,
    pub(crate) hooks: RenderHooks<'a, T>,
    pub(crate) search_term: Option<SearchTerm>,
}

impl<'a, T: ToJsonTreeValue> JsonTreeRenderer<'a, T> {
    pub(crate) fn render_key<'b>(&mut self, ui: &mut Ui, context: RenderKeyContext<'a, 'b>) {
        let response = self.render_key_hook(ui, &context);
        self.response_hook(response, context.pointer);
    }

    pub(crate) fn render_value<'b>(&mut self, ui: &mut Ui, context: RenderValueContext<'a, 'b, T>) {
        let response = self.render_value_hook(ui, &context);
        self.post_render_value_hook(ui, &context);
        self.response_hook(response, context.pointer);
    }

    pub(crate) fn render_punc<'b>(&mut self, ui: &mut Ui, context: RenderPuncContext<'a, 'b>) {
        let response = render_punc(ui, &self.style, context.punc.as_ref());
        if matches!(context.punc, Punc::CollapsedDelimiter(_)) {
            self.response_hook(Some(response), context.pointer);
        }
    }

    fn render_key_hook<'b>(
        &mut self,
        ui: &mut Ui,
        context: &RenderKeyContext<'a, 'b>,
    ) -> Option<Response> {
        if let Some(render_key_hook) = self.hooks.render_key_hook.as_mut() {
            render_key_hook(ui, context)
        } else {
            Some(render_key(
                ui,
                &self.style,
                &context.key,
                self.search_term.as_ref(),
            ))
        }
    }

    fn render_value_hook<'b>(
        &mut self,
        ui: &mut Ui,
        context: &RenderValueContext<'a, 'b, T>,
    ) -> Option<Response> {
        if let Some(render_value_hook) = self.hooks.render_value_hook.as_mut() {
            render_value_hook(ui, context)
        } else {
            Some(render_value(
                ui,
                &self.style,
                &context.display_value.to_string(),
                &context.value_type,
                self.search_term.as_ref(),
            ))
        }
    }

    fn post_render_value_hook<'b>(&mut self, ui: &mut Ui, context: &RenderValueContext<'a, 'b, T>) {
        if let Some(post_render_value_hook) = self.hooks.post_render_value_hook.as_mut() {
            post_render_value_hook(ui, context);
        }
    }

    fn response_hook<'b>(&mut self, response: Option<Response>, pointer: JsonPointer<'a, 'b>) {
        if let (Some(response_hook), Some(response)) =
            (self.hooks.response_hook.as_mut(), response.as_ref())
        {
            response_hook(ResponseContext {
                response: response.clone(),
                pointer,
            })
        }
        // Deprecated.
        if let (Some(response_callback), Some(response)) =
            (self.hooks.response_callback.as_mut(), response)
        {
            response_callback(response, &pointer.to_string())
        }
    }
}

#[derive(Default)]
struct ValueLayoutJobCreator;

impl ValueLayoutJobCreator {
    fn create(
        &self,
        style: &JsonTreeStyle,
        value_str: &str,
        value_type: &BaseValueType,
        search_term: Option<&SearchTerm>,
        font_id: &FontId,
    ) -> LayoutJob {
        let color = style.get_color(value_type);
        let add_quote_if_string = |job: &mut LayoutJob| {
            if *value_type == BaseValueType::String {
                append(job, "\"", color, None, font_id)
            };
        };
        let mut job = LayoutJob::default();
        add_quote_if_string(&mut job);
        add_text_with_highlighting(
            &mut job,
            value_str,
            color,
            search_term,
            style.highlight_color,
            font_id,
        );
        add_quote_if_string(&mut job);
        job
    }
}

impl
    ComputerMut<
        (
            &JsonTreeStyle,
            &str,
            &BaseValueType,
            Option<&SearchTerm>,
            &FontId,
        ),
        LayoutJob,
    > for ValueLayoutJobCreator
{
    fn compute(
        &mut self,
        (style, value_str, value_type, search_term, font_id): (
            &JsonTreeStyle,
            &str,
            &BaseValueType,
            Option<&SearchTerm>,
            &FontId,
        ),
    ) -> LayoutJob {
        self.create(style, value_str, value_type, search_term, font_id)
    }
}

type ValueLayoutJobCreatorCache = FrameCache<LayoutJob, ValueLayoutJobCreator>;

fn render_value(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    value_str: &str,
    value_type: &BaseValueType,
    search_term: Option<&SearchTerm>,
) -> Response {
    let job = ui.ctx().memory_mut(|mem| {
        mem.caches.cache::<ValueLayoutJobCreatorCache>().get((
            style,
            value_str,
            value_type,
            search_term,
            &style.font_id(ui),
        ))
    });

    render_job(ui, job)
}

#[derive(Default)]
struct KeyLayoutJobCreator;

impl KeyLayoutJobCreator {
    fn create(
        &self,
        style: &JsonTreeStyle,
        key: &NestedProperty,
        search_term: Option<&SearchTerm>,
        font_id: &FontId,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        match key {
            NestedProperty::Index(_) => add_array_idx(
                &mut job,
                &key.to_string(),
                style.array_idx_color,
                style.punctuation_color,
                font_id,
            ),
            NestedProperty::Key(_) => add_object_key(
                &mut job,
                &key.to_string(),
                style.object_key_color,
                style.punctuation_color,
                search_term,
                style.highlight_color,
                font_id,
            ),
        };
        job
    }
}

impl<'a>
    ComputerMut<
        (
            &JsonTreeStyle,
            &NestedProperty<'a>,
            Option<&SearchTerm>,
            &FontId,
        ),
        LayoutJob,
    > for KeyLayoutJobCreator
{
    fn compute(
        &mut self,
        (style, parent, search_term, font_id): (
            &JsonTreeStyle,
            &NestedProperty,
            Option<&SearchTerm>,
            &FontId,
        ),
    ) -> LayoutJob {
        self.create(style, parent, search_term, font_id)
    }
}

type KeyLayoutJobCreatorCache = FrameCache<LayoutJob, KeyLayoutJobCreator>;

fn render_key(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    key: &NestedProperty,
    search_term: Option<&SearchTerm>,
) -> Response {
    let job = ui.ctx().memory_mut(|mem| {
        mem.caches.cache::<KeyLayoutJobCreatorCache>().get((
            style,
            key,
            search_term,
            &style.font_id(ui),
        ))
    });

    render_job(ui, job)
}

fn add_object_key(
    job: &mut LayoutJob,
    key_str: &str,
    color: Color32,
    punctuation_color: Color32,
    search_term: Option<&SearchTerm>,
    highlight_color: Color32,
    font_id: &FontId,
) {
    append(job, "\"", color, None, font_id);
    add_text_with_highlighting(job, key_str, color, search_term, highlight_color, font_id);
    append(job, "\"", color, None, font_id);
    append(job, ": ", punctuation_color, None, font_id);
}

fn add_array_idx(
    job: &mut LayoutJob,
    idx_str: &str,
    color: Color32,
    punctuation_color: Color32,
    font_id: &FontId,
) {
    append(job, idx_str, color, None, font_id);
    append(job, ": ", punctuation_color, None, font_id);
}

fn add_text_with_highlighting(
    job: &mut LayoutJob,
    text_str: &str,
    text_color: Color32,
    search_term: Option<&SearchTerm>,
    highlight_color: Color32,
    font_id: &FontId,
) {
    if let Some(search_term) = search_term {
        let matches = search_term.find_match_indices_in(text_str);
        if !matches.is_empty() {
            let mut start = 0;
            for match_idx in matches {
                append(job, &text_str[start..match_idx], text_color, None, font_id);

                let highlight_end_idx = match_idx + search_term.len();

                append(
                    job,
                    &text_str[match_idx..highlight_end_idx],
                    text_color,
                    Some(highlight_color),
                    font_id,
                );

                start = highlight_end_idx;
            }
            append(job, &text_str[start..], text_color, None, font_id);
            return;
        }
    }
    append(job, text_str, text_color, None, font_id);
}

fn append(
    job: &mut LayoutJob,
    text_str: &str,
    color: Color32,
    background_color: Option<Color32>,
    font_id: &FontId,
) {
    let mut text_format = TextFormat {
        color,
        font_id: font_id.clone(),
        ..Default::default()
    };

    if let Some(background_color) = background_color {
        text_format.background = background_color;
    }

    job.append(text_str, 0.0, text_format);
}

fn render_punc(ui: &mut Ui, style: &JsonTreeStyle, punc_str: &str) -> Response {
    let mut job = LayoutJob::default();
    append(
        &mut job,
        punc_str,
        style.punctuation_color,
        None,
        &style.font_id(ui),
    );
    render_job(ui, job)
}

fn render_job(ui: &mut Ui, job: LayoutJob) -> Response {
    ui.add(Label::new(job).sense(Sense::click_and_drag()))
}
