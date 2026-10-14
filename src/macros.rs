//! Internal `macro_rules!` helpers shared by every widget.
//!
//! * [`widget_impls!`] – `Widget for T` and `Widget for &T` from one inherent
//!   `render_into(&self, area, buf)` method.
//! * [`setter!`] / [`opt_setter!`] – consuming builder setters.
//! * [`theme_setter!`] / [`base_style_setter!`] – the `theme` / `base_style` pair every widget has.
//! * [`block_setter!`] – the `block` setter.

/// Implement `ratatui::widgets::Widget` for `T` and `&T`, delegating to the
/// inherent method `fn render_into(&self, area: Rect, buf: &mut Buffer)`.
macro_rules! widget_impls {
    ($t:ty) => {
        impl ::ratatui::widgets::Widget for &$t {
            fn render(self, area: ::ratatui::layout::Rect, buf: &mut ::ratatui::buffer::Buffer) {
                self.render_into(area, buf);
            }
        }

        impl ::ratatui::widgets::Widget for $t {
            fn render(self, area: ::ratatui::layout::Rect, buf: &mut ::ratatui::buffer::Buffer) {
                self.render_into(area, buf);
            }
        }
    };
}

/// Consuming builder setter: `setter!(name: Ty)` assigns `self.name`;
/// `setter!(name => field: Ty)` assigns `self.field`.
macro_rules! setter {
    ($(#[$m:meta])* $name:ident: $ty:ty) => {
        setter!($(#[$m])* $name => $name: $ty);
    };
    ($(#[$m:meta])* $name:ident => $field:ident: $ty:ty) => {
        $(#[$m])*
        pub fn $name(mut self, v: $ty) -> Self {
            self.$field = v;
            self
        }
    };
}

/// Like [`setter!`] but stores `Some(v)` into an `Option<Ty>` field.
macro_rules! opt_setter {
    ($(#[$m:meta])* $name:ident: $ty:ty) => {
        opt_setter!($(#[$m])* $name => $name: $ty);
    };
    ($(#[$m:meta])* $name:ident => $field:ident: $ty:ty) => {
        $(#[$m])*
        pub fn $name(mut self, v: $ty) -> Self {
            self.$field = Some(v);
            self
        }
    };
}

/// Generate the plain `theme(impl Into<Theme>)` setter storing
/// `Some(theme)` in `$field`; attributes (docs) go on the method. Widgets
/// whose `theme` also themes axes write it by hand.
macro_rules! theme_setter {
    ($(#[$m:meta])* $field:ident) => {
        $(#[$m])*
        pub fn theme(mut self, theme: impl Into<$crate::Theme>) -> Self {
            self.$field = Some(theme.into());
            self
        }
    };
}

/// Generate `base_style(Style)` storing `Some(style)` in `$base`.
macro_rules! base_style_setter {
    ($base:ident) => {
        /// Base style painted over the whole area (beats the theme).
        pub fn base_style(mut self, style: ::ratatui::style::Style) -> Self {
            self.$base = Some(style);
            self
        }
    };
}

/// Generate `block(Block)` for a field of type `Option<Block<$lt>>`.
macro_rules! block_setter {
    ($field:ident, $lt:lifetime) => {
        /// Wrap the chart in a [`Block`](ratatui::widgets::Block).
        pub fn block(mut self, b: ::ratatui::widgets::Block<$lt>) -> Self {
            self.$field = Some(b);
            self
        }
    };
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

    #[derive(Default)]
    struct Demo {
        n: u32,
        o: Option<u32>,
        t: Option<crate::Theme>,
        b: Option<Style>,
        blk: Option<ratatui::widgets::Block<'static>>,
    }

    impl Demo {
        setter!(
            /// n
            n: u32
        );
        opt_setter!(
            /// o
            other => o: u32
        );
        theme_setter!(t);
        base_style_setter!(b);
        block_setter!(blk, 'static);

        fn render_into(&self, area: Rect, buf: &mut Buffer) {
            if area.area() > 0 {
                buf[(area.x, area.y)].set_char('x');
            }
        }
    }
    widget_impls!(Demo);

    #[test]
    fn setters_assign() {
        let d = Demo::default().n(3).other(4);
        assert_eq!((d.n, d.o), (3, Some(4)));
    }

    #[test]
    fn theme_setters_assign() {
        let d = Demo::default()
            .theme(crate::Theme::nord())
            .base_style(Style::default());
        assert!(d.t.is_some() && d.b.is_some());
        let d = d.block(ratatui::widgets::Block::bordered());
        assert!(d.blk.is_some());
    }

    #[test]
    fn widget_impls_both_forms() {
        let a = Rect::new(0, 0, 2, 2);
        let mut b1 = Buffer::empty(a);
        let mut b2 = Buffer::empty(a);
        (&Demo::default()).render(a, &mut b1);
        Demo::default().render(a, &mut b2);
        assert_eq!(b1, b2);
        assert_eq!(b1[(0, 0)].symbol(), "x");
    }
}
