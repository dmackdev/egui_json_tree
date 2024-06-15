use std::fmt::Display;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, FontId, Label, Response, Sense, TextFormat, Ui,
};

use crate::{
    delimiters::{ExpandablePunc, SpacingPunc},
    pointer::{JsonPointer, JsonPointerSegment},
    search::SearchTerm,
    value::{BaseValueType, ToJsonTreeValue},
    JsonTreeStyle,
};

type RenderHook<'a, T> = dyn FnMut(&mut Ui, RenderContext<'a, '_, '_, T>) + 'a;

pub trait DefaultRender {
    fn render_default(&self, ui: &mut Ui) -> Response;
}

pub enum RenderContext<'a, 'b, 'c, T: ToJsonTreeValue> {
    Key(&'c RenderKeyContext<'a, 'b>),
    Value(&'c RenderValueContext<'a, 'b, T>),
    ExpandablePunc(&'c RenderExpandablePuncContext<'a, 'b>),
}

impl<'a, 'b, 'c, T: ToJsonTreeValue> DefaultRender for RenderContext<'a, 'b, 'c, T> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        match self {
            RenderContext::Key(context) => context.render_default(ui),
            RenderContext::Value(context) => context.render_default(ui),
            RenderContext::ExpandablePunc(context) => context.render_default(ui),
        }
    }
}

impl<'a, 'b, 'c, T: ToJsonTreeValue> RenderContext<'a, 'b, 'c, T> {
    pub fn pointer(&self) -> JsonPointer {
        match self {
            RenderContext::Key(context) => context.pointer,
            RenderContext::Value(context) => context.pointer,
            RenderContext::ExpandablePunc(context) => context.pointer,
        }
    }
}

pub struct RenderKeyContext<'a, 'b> {
    pub key: JsonPointerSegment<'a>,
    pub pointer: JsonPointer<'a, 'b>,
    pub style: &'b JsonTreeStyle,
    pub(crate) search_term: Option<&'b SearchTerm>,
}

impl<'a, 'b> DefaultRender for RenderKeyContext<'a, 'b> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_key(ui, self.style, &self.key, self.search_term)
    }
}

pub struct RenderValueContext<'a, 'b, T: ToJsonTreeValue> {
    pub value: &'a T,
    pub display_value: &'a dyn Display,
    pub value_type: BaseValueType,
    pub pointer: JsonPointer<'a, 'b>,
    pub style: &'b JsonTreeStyle,
    pub(crate) search_term: Option<&'b SearchTerm>,
}

impl<'a, 'b, T: ToJsonTreeValue> DefaultRender for RenderValueContext<'a, 'b, T> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_value(
            ui,
            self.style,
            &self.display_value.to_string(),
            &self.value_type,
            self.search_term,
        )
    }
}

pub struct RenderExpandablePuncContext<'a, 'b> {
    pub punc: ExpandablePunc,
    pub pointer: JsonPointer<'a, 'b>,
    pub style: &'b JsonTreeStyle,
}

impl<'a, 'b> DefaultRender for RenderExpandablePuncContext<'a, 'b> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_punc(ui, self.style, self.punc.as_ref())
    }
}

pub(crate) struct RenderSpacingPuncContext<'b> {
    pub(crate) punc: SpacingPunc,
    pub(crate) style: &'b JsonTreeStyle,
}

impl<'b> DefaultRender for RenderSpacingPuncContext<'b> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_punc(ui, self.style, self.punc.as_ref())
    }
}

pub(crate) struct RenderHooks<'a, T: ToJsonTreeValue> {
    pub(crate) render_hook: Option<Box<RenderHook<'a, T>>>,
}

impl<'a, T: ToJsonTreeValue> Default for RenderHooks<'a, T> {
    fn default() -> Self {
        Self { render_hook: None }
    }
}

pub(crate) struct JsonTreeRenderer<'a, T: ToJsonTreeValue> {
    pub(crate) hooks: RenderHooks<'a, T>,
}

impl<'a, T: ToJsonTreeValue> JsonTreeRenderer<'a, T> {
    pub(crate) fn render_key<'b>(&mut self, ui: &mut Ui, context: RenderKeyContext<'a, 'b>) {
        match self.hooks.render_hook.as_mut() {
            Some(render_hook) => {
                render_hook(ui, RenderContext::Key(&context));
            }
            None => {
                context.render_default(ui);
            }
        };
    }

    pub(crate) fn render_value<'b>(&mut self, ui: &mut Ui, context: RenderValueContext<'a, 'b, T>) {
        match self.hooks.render_hook.as_mut() {
            Some(render_hook) => {
                render_hook(ui, RenderContext::Value(&context));
            }
            None => {
                context.render_default(ui);
            }
        };
    }

    pub(crate) fn render_expandable_punc<'b>(
        &mut self,
        ui: &mut Ui,
        context: RenderExpandablePuncContext<'a, 'b>,
    ) {
        match self.hooks.render_hook.as_mut() {
            Some(render_hook) => {
                render_hook(ui, RenderContext::ExpandablePunc(&context));
            }
            None => {
                context.render_default(ui);
            }
        };
    }

    pub(crate) fn render_spacing_punc(&mut self, ui: &mut Ui, context: RenderSpacingPuncContext) {
        context.render_default(ui);
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
        key: &JsonPointerSegment,
        search_term: Option<&SearchTerm>,
        font_id: &FontId,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        match key {
            JsonPointerSegment::Index(_) => {
                add_array_idx(&mut job, &key.to_string(), style.array_idx_color, font_id)
            }
            JsonPointerSegment::Key(_) => add_object_key(
                &mut job,
                &key.to_string(),
                style.object_key_color,
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
            &JsonPointerSegment<'a>,
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
            &JsonPointerSegment,
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
    key: &JsonPointerSegment,
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
    search_term: Option<&SearchTerm>,
    highlight_color: Color32,
    font_id: &FontId,
) {
    append(job, "\"", color, None, font_id);
    add_text_with_highlighting(job, key_str, color, search_term, highlight_color, font_id);
    append(job, "\"", color, None, font_id);
}

fn add_array_idx(job: &mut LayoutJob, idx_str: &str, color: Color32, font_id: &FontId) {
    append(job, idx_str, color, None, font_id);
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
