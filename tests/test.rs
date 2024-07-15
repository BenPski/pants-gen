#[cfg(test)]
mod tests {
    use pants_gen::{charset::Charset, choice::Choice, password::PasswordSpec};

    #[test]
    fn default_spec_parses() {
        let spec_string = PasswordSpec::default().to_string();
        let spec = spec_string.parse::<PasswordSpec>();
        assert!(spec.is_ok());
    }

    #[test]
    fn default_spec_works() {
        let spec = PasswordSpec::default();
        let gen = spec.generate();
        assert!(gen.is_some());
    }

    #[test]
    fn parse_choice() {
        let choice_string = Charset::Upper.exactly(10).to_string();
        let parsed = choice_string.parse::<Choice>();
        assert!(parsed.is_ok());
    }

    // should have a better test for these since there are quite a few combinations
    #[test]
    fn correct_length() {
        let length = 10;
        let spec = PasswordSpec::default().length(length);
        let gen = spec.generate().map(|s| s.len());
        assert_eq!(Some(length), gen);
    }

    #[test]
    fn exactly_check() {
        let amount = 5;
        let spec = PasswordSpec::default().upper_exactly(amount);
        let gen = spec
            .generate()
            .map(|s| s.chars().filter(|c| c.is_uppercase()).count());

        assert_eq!(Some(amount), gen);
    }

    #[test]
    fn at_most_check() {
        let amount = 5;
        let spec = PasswordSpec::default().lower_at_most(amount);
        let gen = spec
            .generate()
            .map(|s| s.chars().filter(|c| c.is_lowercase()).count())
            .unwrap();

        assert!(gen <= amount);
    }

    #[test]
    fn at_least_check() {
        let amount = 5;
        let spec = PasswordSpec::default().number_at_least(amount);
        let gen = spec
            .generate()
            .map(|s| s.chars().filter(|c| c.is_ascii_digit()).count())
            .unwrap();

        assert!(gen >= amount);
    }

    #[test]
    fn bad_interval() {
        let spec_string = "32//1-0|:upper:";
        let spec = spec_string.parse::<PasswordSpec>();
        assert!(spec.is_err())
    }

    #[test]
    fn bad_pattern() {
        let spec_string = "32//1+|:not_real:";
        let spec = spec_string.parse::<PasswordSpec>();
        assert!(spec.is_err())
    }

    #[test]
    fn bad_length() {
        let spec_string = "twenty//1+|:upper:";
        let spec = spec_string.parse::<PasswordSpec>();
        assert!(spec.is_err())
    }
}
