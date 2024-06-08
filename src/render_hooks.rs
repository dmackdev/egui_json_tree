use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, FontId, Label, Response, Sense, TextFormat, Ui,
};

use crate::{
    delimiters::Punc,
    search::SearchTerm,
    value::{BaseValueType, ExpandableType, Parent},
    JsonTreeStyle,
};

type ResponseCallback<'a> = dyn FnMut(Response, &String) + 'a;

#[derive(Default)]
pub(crate) struct RenderHooks<'a> {
    pub(crate) style: JsonTreeStyle,
    pub(crate) response_callback: Option<Box<ResponseCallback<'a>>>,
}

impl<'a> RenderHooks<'a> {
    pub(crate) fn render_key(
        &mut self,
        ui: &mut Ui,
        parent: &Parent,
        search_term: Option<&SearchTerm>,
        pointer_str: &String,
    ) {
        let response = render_key(ui, &self.style, parent, search_term);
        self.response_callback(response, pointer_str);
    }

    pub(crate) fn render_value(
        &mut self,
        ui: &mut Ui,
        value_str: &str,
        value_type: &BaseValueType,
        search_term: Option<&SearchTerm>,
        pointer_str: &String,
    ) {
        let response = render_value(ui, &self.style, value_str, value_type, search_term);
        self.response_callback(response, pointer_str);
    }

    pub(crate) fn render_punc(&mut self, ui: &mut Ui, punc: &Punc, pointer_str: &String) {
        let response = render_punc(ui, &self.style, punc.as_ref());
        if matches!(punc, Punc::CollapsedDelimiter(_)) {
            self.response_callback(response, pointer_str);
        }
    }

    fn response_callback(&mut self, response: Response, pointer_str: &String) {
        if let Some(response_callback) = self.response_callback.as_mut() {
            response_callback(response, pointer_str)
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
        parent: &Parent,
        search_term: Option<&SearchTerm>,
        font_id: &FontId,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        match parent {
            Parent {
                key,
                expandable_type: ExpandableType::Array,
            } => add_array_idx(
                &mut job,
                key,
                style.array_idx_color,
                style.punctuation_color,
                font_id,
            ),
            Parent {
                key,
                expandable_type: ExpandableType::Object,
            } => add_object_key(
                &mut job,
                key,
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

impl ComputerMut<(&JsonTreeStyle, &Parent, Option<&SearchTerm>, &FontId), LayoutJob>
    for KeyLayoutJobCreator
{
    fn compute(
        &mut self,
        (style, parent, search_term, font_id): (
            &JsonTreeStyle,
            &Parent,
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
    parent: &Parent,
    search_term: Option<&SearchTerm>,
) -> Response {
    let job = ui.ctx().memory_mut(|mem| {
        mem.caches.cache::<KeyLayoutJobCreatorCache>().get((
            style,
            parent,
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
