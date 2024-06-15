use std::fmt::Display;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, FontId, Label, Response, Sense, TextFormat, Ui,
};

use crate::{
    delimiters::{ExpandableDelimiter, SpacingDelimiter},
    pointer::{JsonPointer, JsonPointerSegment},
    search::SearchTerm,
    value::{BaseValueType, ToJsonTreeValue},
    JsonTreeStyle,
};

pub type RenderHook<'a, T> = dyn FnMut(&mut Ui, RenderContext<'a, '_, T>) + 'a;

pub trait DefaultRender {
    fn render_default(&self, ui: &mut Ui) -> Response;
}

#[derive(Clone, Copy)]
pub enum RenderContext<'a, 'b, T: ToJsonTreeValue> {
    Property(RenderPropertyContext<'a, 'b, T>),
    Value(RenderValueContext<'a, 'b, T>),
    ExpandableDelimiter(RenderExpandableDelimiterContext<'a, 'b, T>),
}

impl<'a, 'b, T: ToJsonTreeValue> DefaultRender for RenderContext<'a, 'b, T> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        match self {
            RenderContext::Property(context) => context.render_default(ui),
            RenderContext::Value(context) => context.render_default(ui),
            RenderContext::ExpandableDelimiter(context) => context.render_default(ui),
        }
    }
}

impl<'a, 'b, T: ToJsonTreeValue> RenderContext<'a, 'b, T> {
    pub fn pointer(&self) -> JsonPointer {
        match self {
            RenderContext::Property(context) => context.pointer,
            RenderContext::Value(context) => context.pointer,
            RenderContext::ExpandableDelimiter(context) => context.pointer,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderPropertyContext<'a, 'b, T: ToJsonTreeValue> {
    pub property: JsonPointerSegment<'a>,
    pub value: &'a T,
    pub pointer: JsonPointer<'a, 'b>,
    pub style: &'b JsonTreeStyle,
    pub(crate) search_term: Option<&'b SearchTerm>,
}

impl<'a, 'b, T: ToJsonTreeValue> DefaultRender for RenderPropertyContext<'a, 'b, T> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_property(ui, self.style, &self.property, self.search_term)
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct RenderExpandableDelimiterContext<'a, 'b, T: ToJsonTreeValue> {
    pub delimiter: ExpandableDelimiter,
    pub value: &'a T,
    pub pointer: JsonPointer<'a, 'b>,
    pub style: &'b JsonTreeStyle,
}

impl<'a, 'b, T: ToJsonTreeValue> DefaultRender for RenderExpandableDelimiterContext<'a, 'b, T> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_delimiter(ui, self.style, self.delimiter.as_ref())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RenderSpacingDelimiterContext<'b> {
    pub(crate) delimiter: SpacingDelimiter,
    pub(crate) style: &'b JsonTreeStyle,
}

impl<'b> DefaultRender for RenderSpacingDelimiterContext<'b> {
    fn render_default(&self, ui: &mut Ui) -> Response {
        render_delimiter(ui, self.style, self.delimiter.as_ref())
    }
}

pub(crate) struct JsonTreeRenderer<'a, T: ToJsonTreeValue> {
    pub(crate) render_hook: Option<Box<RenderHook<'a, T>>>,
}

impl<'a, T: ToJsonTreeValue> Default for JsonTreeRenderer<'a, T> {
    fn default() -> Self {
        Self { render_hook: None }
    }
}

impl<'a, T: ToJsonTreeValue> JsonTreeRenderer<'a, T> {
    pub(crate) fn render_property<'b>(
        &mut self,
        ui: &mut Ui,
        context: RenderPropertyContext<'a, 'b, T>,
    ) {
        match self.render_hook.as_mut() {
            Some(render_hook) => {
                render_hook(ui, RenderContext::Property(context));
            }
            None => {
                context.render_default(ui);
            }
        };
    }

    pub(crate) fn render_value<'b>(&mut self, ui: &mut Ui, context: RenderValueContext<'a, 'b, T>) {
        match self.render_hook.as_mut() {
            Some(render_hook) => {
                render_hook(ui, RenderContext::Value(context));
            }
            None => {
                context.render_default(ui);
            }
        };
    }

    pub(crate) fn render_expandable_delimiter<'b>(
        &mut self,
        ui: &mut Ui,
        context: RenderExpandableDelimiterContext<'a, 'b, T>,
    ) {
        match self.render_hook.as_mut() {
            Some(render_hook) => {
                render_hook(ui, RenderContext::ExpandableDelimiter(context));
            }
            None => {
                context.render_default(ui);
            }
        };
    }

    pub(crate) fn render_spacing_delimiter(
        &mut self,
        ui: &mut Ui,
        context: RenderSpacingDelimiterContext,
    ) {
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
struct PropertyLayoutJobCreator;

impl PropertyLayoutJobCreator {
    fn create(
        &self,
        style: &JsonTreeStyle,
        property: &JsonPointerSegment,
        search_term: Option<&SearchTerm>,
        font_id: &FontId,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        match property {
            JsonPointerSegment::Index(_) => add_array_idx(
                &mut job,
                &property.to_string(),
                style.array_idx_color,
                font_id,
            ),
            JsonPointerSegment::Key(_) => add_object_key(
                &mut job,
                &property.to_string(),
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
    > for PropertyLayoutJobCreator
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

type PropertyLayoutJobCreatorCache = FrameCache<LayoutJob, PropertyLayoutJobCreator>;

fn render_property(
    ui: &mut Ui,
    style: &JsonTreeStyle,
    property: &JsonPointerSegment,
    search_term: Option<&SearchTerm>,
) -> Response {
    let job = ui.ctx().memory_mut(|mem| {
        mem.caches.cache::<PropertyLayoutJobCreatorCache>().get((
            style,
            property,
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

fn render_delimiter(ui: &mut Ui, style: &JsonTreeStyle, delimiter_str: &str) -> Response {
    let mut job = LayoutJob::default();
    append(
        &mut job,
        delimiter_str,
        style.punctuation_color,
        None,
        &style.font_id(ui),
    );
    render_job(ui, job)
}

fn render_job(ui: &mut Ui, job: LayoutJob) -> Response {
    ui.add(Label::new(job).sense(Sense::click_and_drag()))
}
