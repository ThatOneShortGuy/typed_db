use std::fmt::Display;

use syn::{
    LitInt, LitStr, Result,
    parse::{Parse, ParseStream},
};

mod kw {
    syn::custom_keyword!(CURRENT_TIMESTAMP);
    syn::custom_keyword!(CURRENT_DATE);
    syn::custom_keyword!(CURRENT_TIME);
    syn::custom_keyword!(TRUE);
    syn::custom_keyword!(FALSE);
}

pub enum DefaultValues {
    CurrentTimestamp,
    CurrentDate,
    CurrentTime,
    True,
    False,
    IntLiteral(LitInt),
    StrLiteral(LitStr),
}

impl Parse for DefaultValues {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::CURRENT_TIMESTAMP) {
            input.parse::<kw::CURRENT_TIMESTAMP>()?;
            Ok(DefaultValues::CurrentTimestamp)
        } else if lookahead.peek(kw::CURRENT_DATE) {
            input.parse::<kw::CURRENT_DATE>()?;
            Ok(DefaultValues::CurrentDate)
        } else if lookahead.peek(kw::CURRENT_TIME) {
            input.parse::<kw::CURRENT_TIME>()?;
            Ok(DefaultValues::CurrentTime)
        } else if lookahead.peek(kw::TRUE) {
            input.parse::<kw::TRUE>()?;
            Ok(DefaultValues::True)
        } else if lookahead.peek(kw::FALSE) {
            input.parse::<kw::FALSE>()?;
            Ok(DefaultValues::False)
        } else if lookahead.peek(LitInt) {
            Ok(DefaultValues::IntLiteral(input.parse()?))
        } else if lookahead.peek(LitStr) {
            Ok(DefaultValues::StrLiteral(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Display for DefaultValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultValues::CurrentTimestamp => write!(f, "CURRENT_TIMESTAMP"),
            DefaultValues::CurrentDate => write!(f, "CURRENT_DATE"),
            DefaultValues::CurrentTime => write!(f, "CURRENT_TIME"),
            DefaultValues::True => write!(f, "TRUE"),
            DefaultValues::False => write!(f, "FALSE"),
            DefaultValues::IntLiteral(lit_int) => write!(f, "{}", lit_int),
            DefaultValues::StrLiteral(lit_str) => write!(f, "{}", lit_str.value()),
        }
    }
}
