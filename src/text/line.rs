#![allow(deprecated)]

use std::rc::Rc;

use super::{Span, Spans, Style, StyledGrapheme};
use crate::layout::Alignment;

#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct Line {
    pub spans: Vec<Span>,
    pub alignment: Option<Alignment>,
}

impl Line {
    /// Create a line with a style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::text::Line;
    /// # use ratatui::style::{Color, Modifier, Style};
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// Line::styled("My text", style);
    /// Line::styled(String::from("My text"), style);
    /// ```
    pub fn styled<T>(content: T, style: Style) -> Line
    where
        T: Into<String>,
    {
        Line::from(Span::styled(content, style))
    }

    /// Returns the width of the underlying string.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use ratatui::text::{Span, Line};
    /// # use ratatui::style::{Color, Style};
    /// let line = Line::from(vec![
    ///     Span::styled("My", Style::default().fg(Color::Yellow)),
    ///     Span::raw(" text"),
    /// ]);
    /// assert_eq!(7, line.width());
    /// ```
    pub fn width(&self) -> usize {
        self.spans.iter().map(Span::width).sum()
    }

    /// Returns an iterator over the graphemes held by this line.
    ///
    /// `base_style` is the [`Style`] that will be patched with each grapheme [`Style`] to get
    /// the resulting [`Style`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use ratatui::text::{Line, StyledGrapheme};
    /// # use ratatui::style::{Color, Modifier, Style};
    /// # use std::iter::Iterator;
    /// # use std::rc::Rc;
    /// let style = Style::default().fg(Color::Yellow);
    /// let line = Line::styled("Text", style);
    /// let style = Style::default().fg(Color::Green).bg(Color::Black);
    /// let styled_graphemes = line.styled_graphemes(style);
    /// assert_eq!(
    ///     vec![
    ///         StyledGrapheme {
    ///             symbol: Rc::new("T".to_owned()),
    ///             style: Style {
    ///                 fg: Some(Color::Yellow),
    ///                 bg: Some(Color::Black),
    ///                 add_modifier: Modifier::empty(),
    ///                 sub_modifier: Modifier::empty(),
    ///             },
    ///         },
    ///         StyledGrapheme {
    ///             symbol: Rc::new("e".to_owned()),
    ///             style: Style {
    ///                 fg: Some(Color::Yellow),
    ///                 bg: Some(Color::Black),
    ///                 add_modifier: Modifier::empty(),
    ///                 sub_modifier: Modifier::empty(),
    ///             },
    ///         },
    ///         StyledGrapheme {
    ///             symbol: Rc::new("x".to_owned()),
    ///             style: Style {
    ///                 fg: Some(Color::Yellow),
    ///                 bg: Some(Color::Black),
    ///                 add_modifier: Modifier::empty(),
    ///                 sub_modifier: Modifier::empty(),
    ///             },
    ///         },
    ///         StyledGrapheme {
    ///             symbol: Rc::new("t".to_owned()),
    ///             style: Style {
    ///                 fg: Some(Color::Yellow),
    ///                 bg: Some(Color::Black),
    ///                 add_modifier: Modifier::empty(),
    ///                 sub_modifier: Modifier::empty(),
    ///             },
    ///         },
    ///     ],
    ///     styled_graphemes.collect::<Vec<StyledGrapheme>>()
    /// );
    /// ```
    pub fn styled_graphemes(&self, base_style: Style) -> impl Iterator<Item = StyledGrapheme> + '_ {
        self.spans
            .iter()
            .flat_map(move |span| span.styled_graphemes(base_style))
    }

    /// Patches the style of each Span in an existing Line, adding modifiers from the given style.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use ratatui::text::{Span, Line};
    /// # use ratatui::style::{Color, Style, Modifier};
    /// let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    /// let mut raw_line = Line::from(vec![
    ///     Span::raw("My"),
    ///     Span::raw(" text"),
    /// ]);
    /// let mut styled_line = Line::from(vec![
    ///     Span::styled("My", style),
    ///     Span::styled(" text", style),
    /// ]);
    ///
    /// assert_ne!(raw_line, styled_line);
    ///
    /// raw_line.patch_style(style);
    /// assert_eq!(raw_line, styled_line);
    /// ```
    pub fn patch_style(&mut self, style: Style) {
        for span in &mut self.spans {
            span.patch_style(style);
        }
    }

    /// Resets the style of each Span in the Line.
    /// Equivalent to calling `patch_style(Style::reset())`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use ratatui::text::{Span, Line};
    /// # use ratatui::style::{Color, Style, Modifier};
    /// let mut line = Line::from(vec![
    ///     Span::styled("My", Style::default().fg(Color::Yellow)),
    ///     Span::styled(" text", Style::default().add_modifier(Modifier::BOLD)),
    /// ]);
    ///
    /// line.reset_style();
    /// assert_eq!(Style::reset(), line.spans[0].style);
    /// assert_eq!(Style::reset(), line.spans[1].style);
    /// ```
    pub fn reset_style(&mut self) {
        for span in &mut self.spans {
            span.reset_style();
        }
    }

    /// Sets the target alignment for this line of text.
    /// Defaults to: [`None`], meaning the alignment is determined by the rendering widget.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use std::borrow::Cow;
    /// # use ratatui::layout::Alignment;
    /// # use ratatui::text::{Span, Line};
    /// # use ratatui::style::{Color, Style, Modifier};
    /// let mut line = Line::from("Hi, what's up?");
    /// assert_eq!(None, line.alignment);
    /// assert_eq!(Some(Alignment::Right), line.alignment(Alignment::Right).alignment)
    /// ```
    pub fn alignment(self, alignment: Alignment) -> Self {
        Self {
            alignment: Some(alignment),
            ..self
        }
    }
}

impl FromIterator<StyledGrapheme> for Line {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = StyledGrapheme>,
    {
        let mut spans = Vec::new();
        // Buffers are used because the content of each Span is not mutable after it is created.
        let mut content_buffer = String::new();
        let mut style_buffer = Style::default();
        for (i, styled_grapheme) in iter.into_iter().enumerate() {
            if i == 0 {
                content_buffer = styled_grapheme.symbol.as_ref().clone();
                style_buffer = styled_grapheme.style;
            } else {
                // If the style of the current grapheme is the same as the previous one,
                // we can just append the grapheme to the previous content.
                // This allows us to avoid creating a new Span for each grapheme.
                if style_buffer == styled_grapheme.style {
                    content_buffer.push_str(&styled_grapheme.symbol);
                    continue;
                } else {
                    spans.push(Span {
                        content: Rc::new(content_buffer.clone()),
                        style: style_buffer,
                    });

                    content_buffer = styled_grapheme.symbol.as_ref().clone();
                    style_buffer = styled_grapheme.style;
                }
            }
        }

        // Push the last Span.
        spans.push(Span {
            content: Rc::new(content_buffer),
            style: style_buffer,
        });

        spans.into()
    }
}

impl From<String> for Line {
    fn from(s: String) -> Self {
        Self::from(vec![Span::from(s)])
    }
}

impl From<&str> for Line {
    fn from(s: &str) -> Self {
        Self::from(vec![Span::from(s)])
    }
}

impl From<Vec<Span>> for Line {
    fn from(spans: Vec<Span>) -> Self {
        Self {
            spans,
            ..Default::default()
        }
    }
}

impl From<Span> for Line {
    fn from(span: Span) -> Self {
        Self::from(vec![span])
    }
}

impl From<Line> for String {
    fn from(line: Line) -> String {
        line.spans.iter().fold(String::new(), |mut acc, s| {
            acc.push_str(s.content.as_ref());
            acc
        })
    }
}

impl From<Spans> for Line {
    fn from(value: Spans) -> Self {
        Self::from(value.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        layout::Alignment,
        style::{Color, Modifier, Style},
        text::{Line, Span, Spans},
    };

    #[test]
    fn test_width() {
        let line = Line::from(vec![
            Span::styled("My", Style::default().fg(Color::Yellow)),
            Span::raw(" text"),
        ]);
        assert_eq!(7, line.width());

        let empty_line = Line::default();
        assert_eq!(0, empty_line.width());
    }

    #[test]
    fn test_patch_style() {
        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC);
        let mut raw_line = Line::from(vec![Span::raw("My"), Span::raw(" text")]);
        let styled_line = Line::from(vec![
            Span::styled("My", style),
            Span::styled(" text", style),
        ]);

        assert_ne!(raw_line, styled_line);

        raw_line.patch_style(style);
        assert_eq!(raw_line, styled_line);
    }

    #[test]
    fn test_reset_style() {
        let mut line = Line::from(vec![
            Span::styled("My", Style::default().fg(Color::Yellow)),
            Span::styled(" text", Style::default().add_modifier(Modifier::BOLD)),
        ]);

        line.reset_style();
        assert_eq!(Style::reset(), line.spans[0].style);
        assert_eq!(Style::reset(), line.spans[1].style);
    }

    #[test]
    fn test_from_string() {
        let s = String::from("Hello, world!");
        let line = Line::from(s);
        assert_eq!(vec![Span::from("Hello, world!")], line.spans);
    }

    #[test]
    fn test_from_str() {
        let s = "Hello, world!";
        let line = Line::from(s);
        assert_eq!(vec![Span::from("Hello, world!")], line.spans);
    }

    #[test]
    fn test_from_vec() {
        let spans = vec![
            Span::styled("Hello,", Style::default().fg(Color::Red)),
            Span::styled(" world!", Style::default().fg(Color::Green)),
        ];
        let line = Line::from(spans.clone());
        assert_eq!(spans, line.spans);
    }

    #[test]
    fn test_from_span() {
        let span = Span::styled("Hello, world!", Style::default().fg(Color::Yellow));
        let line = Line::from(span.clone());
        assert_eq!(vec![span], line.spans);
    }

    #[test]
    fn test_from_spans() {
        let spans = vec![
            Span::styled("Hello,", Style::default().fg(Color::Red)),
            Span::styled(" world!", Style::default().fg(Color::Green)),
        ];
        assert_eq!(Line::from(Spans::from(spans.clone())), Line::from(spans));
    }

    #[test]
    fn test_into_string() {
        let line = Line::from(vec![
            Span::styled("Hello,", Style::default().fg(Color::Red)),
            Span::styled(" world!", Style::default().fg(Color::Green)),
        ]);
        let s: String = line.into();
        assert_eq!("Hello, world!", s);
    }

    #[test]
    fn test_alignment() {
        let line = Line::from("This is left").alignment(Alignment::Left);
        assert_eq!(Some(Alignment::Left), line.alignment);

        let line = Line::from("This is default");
        assert_eq!(None, line.alignment);
    }
}
