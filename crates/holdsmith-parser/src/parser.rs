use smol_str::SmolStr;

use crate::{
    lexer::{lex, SpannedToken, Token},
    AddCardEffect, AssignOp, Choice, ChronicleEffect, CompareOp, Condition, ConditionClause,
    Context, DamageEffect, Effect, FlagCondition, FlagEffect, FlagValue, Frontmatter,
    NavigationTarget, ParseError, Passage, PassageContent, Prose, RemoveCardsEffect,
    ReputationEffect, Requirements, ResourceCheck, ResourceCondition, ResourceEffect, RhaiBlock,
    SceneFile, ScriptEffect, Span, TagCondition, TagSource,
};

const RESOURCES: &[&str] = &["credits", "fuel", "supplies", "hull", "morale", "integrity"];

fn is_resource(s: &str) -> bool {
    RESOURCES.contains(&s)
}

pub struct Parser {
    source: String,
    tokens: Vec<SpannedToken>,
    pos: usize,
    #[allow(dead_code)]
    filename: SmolStr,
}

impl Parser {
    pub fn new(source: &str, filename: &str) -> Self {
        let tokens = lex(source);
        Self {
            source: source.to_string(),
            tokens,
            pos: 0,
            filename: SmolStr::new(filename),
        }
    }

    fn current(&self) -> &SpannedToken {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens.last().expect("Token stream should not be empty")
        })
    }

    fn current_token(&self) -> &Token {
        &self.current().token
    }

    fn current_span(&self) -> Span {
        self.current().span.clone()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(token)
    }

    #[allow(dead_code)]
    fn check_identifier(&self) -> bool {
        matches!(self.current_token(), Token::Identifier(_))
    }

    fn advance(&mut self) -> &SpannedToken {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn consume(&mut self, expected: &Token, message: &str) -> Result<SpannedToken, ParseError> {
        if self.check(expected) {
            Ok(self.advance().clone())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{}", self.current_token()),
                span: self.current_span(),
            })
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&Token::Newline) {}
    }

    pub fn parse(&mut self) -> Result<SceneFile, ParseError> {
        self.skip_newlines();

        let frontmatter = self.parse_frontmatter()?;
        let mut passages = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            if matches!(self.current_token(), Token::PassageHeader(_)) {
                passages.push(self.parse_passage()?);
            } else {
                self.advance();
            }
        }

        let span = 0..self.tokens.last().map(|t| t.span.end).unwrap_or(0);

        Ok(SceneFile {
            frontmatter,
            passages,
            span,
        })
    }

    fn parse_frontmatter(&mut self) -> Result<Frontmatter, ParseError> {
        let start_span = self.current_span();
        let start_token = self.consume(&Token::FrontmatterDelim, "Expected --- to start frontmatter")?;

        // Find the end of frontmatter by looking for the next ---
        // Skip tokens until we find the closing ---
        let yaml_start = start_token.span.end;
        let mut yaml_end = yaml_start;

        while !self.check(&Token::FrontmatterDelim) && !self.is_at_end() {
            yaml_end = self.current_span().end;
            self.advance();
        }

        let end_span = self.current_span();
        self.consume(&Token::FrontmatterDelim, "Expected --- to end frontmatter")?;
        self.skip_newlines();

        // Extract YAML from raw source
        let yaml_str = &self.source[yaml_start..yaml_end];
        let yaml_data: serde_yaml::Value = serde_yaml::from_str(&yaml_str).map_err(|e| {
            ParseError::InvalidFrontmatter {
                message: e.to_string(),
                span: start_span.clone(),
            }
        })?;

        let id = yaml_data
            .get("id")
            .and_then(|v| v.as_str())
            .map(SmolStr::new)
            .unwrap_or_default();

        let title = yaml_data
            .get("title")
            .and_then(|v| v.as_str())
            .map(SmolStr::new)
            .unwrap_or_default();

        let tags = yaml_data
            .get("tags")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(SmolStr::new))
                    .collect()
            })
            .unwrap_or_default();

        let context = match yaml_data.get("context").and_then(|v| v.as_str()) {
            Some("journey") => Context::Journey,
            Some("port") => Context::Port,
            _ => Context::Any,
        };

        let weight = yaml_data
            .get("weight")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;

        let cooldown = yaml_data
            .get("cooldown")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as u32;

        let requires = yaml_data
            .get("requires")
            .and_then(|v| v.as_mapping())
            .map(|m| self.parse_requirements_from_yaml(m, &start_span));

        let span = start_span.start..end_span.end;

        Ok(Frontmatter {
            id,
            title,
            tags,
            context,
            weight,
            cooldown,
            requires,
            span,
        })
    }

    #[allow(dead_code)]
    fn collect_line(&mut self) -> String {
        let mut parts = Vec::new();

        while !self.check(&Token::Newline)
            && !self.check(&Token::FrontmatterDelim)
            && !self.is_at_end()
        {
            let part = match self.current_token() {
                Token::Identifier(s) => s.to_string(),
                Token::Number(n) => n.to_string(),
                Token::Float(n) => n.to_string(),
                Token::String(s) => format!("\"{}\"", s),
                Token::Text(s) => s.to_string(),
                Token::Colon => ":".to_string(),
                Token::Comma => ",".to_string(),
                Token::LBracket => "[".to_string(),
                Token::RBracket => "]".to_string(),
                Token::LBrace => "{".to_string(),
                Token::RBrace => "}".to_string(),
                Token::Dot => ".".to_string(),
                Token::Eq => "=".to_string(),
                _ => String::new(),
            };
            if !part.is_empty() {
                parts.push(part);
            }
            self.advance();
        }

        parts.join(" ")
    }

    fn parse_requirements_from_yaml(
        &self,
        map: &serde_yaml::Mapping,
        span: &Span,
    ) -> Requirements {
        let get_string_array = |key: &str| -> Vec<SmolStr> {
            map.get(key)
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(SmolStr::new))
                        .collect()
                })
                .unwrap_or_default()
        };

        let get_resource_checks = |key: &str| -> Vec<ResourceCheck> {
            map.get(key)
                .and_then(|v| v.as_mapping())
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| {
                            let resource = k.as_str().map(SmolStr::new)?;
                            let value = v.as_i64()?;
                            Some(ResourceCheck { resource, value })
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Requirements {
            ship_tags: get_string_array("shipTags"),
            crew_tags: get_string_array("crewTags"),
            cargo_tags: get_string_array("cargoTags"),
            min_resources: get_resource_checks("minResources"),
            max_resources: get_resource_checks("maxResources"),
            required_flags: get_string_array("requiredFlags"),
            excluded_flags: get_string_array("excludedFlags"),
            span: span.clone(),
        }
    }

    fn parse_passage(&mut self) -> Result<Passage, ParseError> {
        let header_token = self.advance().clone();
        let name = match &header_token.token {
            Token::PassageHeader(name) => name.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "passage header".to_string(),
                    found: format!("{}", header_token.token),
                    span: header_token.span,
                })
            }
        };

        self.skip_newlines();

        let mut content = Vec::new();
        let mut prose_lines = Vec::new();
        let mut prose_start: Option<Span> = None;

        let flush_prose = |content: &mut Vec<PassageContent>,
                          prose_lines: &mut Vec<String>,
                          prose_start: &mut Option<Span>| {
            if !prose_lines.is_empty() {
                let text = prose_lines.join("\n").trim().to_string();
                if !text.is_empty() {
                    content.push(PassageContent::Prose(Prose {
                        text: SmolStr::new(&text),
                        span: prose_start.clone().unwrap_or(0..0),
                    }));
                }
                prose_lines.clear();
                *prose_start = None;
            }
        };

        while !self.is_at_end() && !matches!(self.current_token(), Token::PassageHeader(_)) {
            self.skip_newlines();

            if self.is_at_end() || matches!(self.current_token(), Token::PassageHeader(_)) {
                break;
            }

            match self.current_token() {
                Token::ChoiceMarker => {
                    flush_prose(&mut content, &mut prose_lines, &mut prose_start);
                    content.push(PassageContent::Choice(self.parse_choice()?));
                }
                Token::LBrace => {
                    // Inline Rhai block at passage level
                    flush_prose(&mut content, &mut prose_lines, &mut prose_start);
                    content.push(PassageContent::RhaiBlock(self.parse_rhai_block()?));
                }
                Token::Identifier(_) | Token::Text(_) => {
                    if prose_start.is_none() {
                        prose_start = Some(self.current_span());
                    }
                    let line = self.collect_prose_line();
                    if !line.is_empty() {
                        prose_lines.push(line);
                    }
                }
                Token::Newline => {
                    self.advance();
                }
                Token::EffectMarker | Token::Arrow => {
                    // These shouldn't appear at passage level outside choices
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }

        flush_prose(&mut content, &mut prose_lines, &mut prose_start);

        let span = header_token.span.start..self.current_span().start;

        Ok(Passage {
            name,
            content,
            span,
        })
    }

    fn collect_prose_line(&mut self) -> String {
        let mut parts = Vec::new();

        while !self.check(&Token::Newline)
            && !self.check(&Token::ChoiceMarker)
            && !self.check(&Token::EffectMarker)
            && !matches!(self.current_token(), Token::PassageHeader(_))
            && !self.is_at_end()
        {
            let part = match self.current_token() {
                Token::Identifier(s) => s.to_string(),
                Token::Text(s) => s.to_string(),
                Token::Number(n) => n.to_string(),
                Token::String(s) => format!("\"{}\"", s),
                Token::Dot => ".".to_string(),
                Token::Comma => ",".to_string(),
                Token::Colon => ":".to_string(),
                Token::Bang => "!".to_string(),
                _ => String::new(),
            };
            if !part.is_empty() {
                parts.push(part);
            }
            self.advance();
        }

        parts.join(" ")
    }

    fn parse_choice(&mut self) -> Result<Choice, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::ChoiceMarker, "Expected * for choice")?;

        // Parse choice text: [text here]
        self.consume(&Token::LBracket, "Expected [ after *")?;

        let mut text_parts = Vec::new();
        while !self.check(&Token::RBracket) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                break;
            }
            let part = match self.current_token() {
                Token::Identifier(s) => s.to_string(),
                Token::Text(s) => s.to_string(),
                Token::Number(n) => n.to_string(),
                Token::String(s) => s.to_string(),
                Token::Dot => ".".to_string(),
                Token::Comma => ",".to_string(),
                Token::Colon => ":".to_string(),
                Token::Bang => "!".to_string(),
                _ => String::new(),
            };
            if !part.is_empty() {
                text_parts.push(part);
            }
            self.advance();
        }
        let text = SmolStr::new(&text_parts.join(" "));
        self.consume(&Token::RBracket, "Expected ] to close choice text")?;

        // Optional condition: { ... } or `when <expr>`
        let condition = if self.check(&Token::LBrace) {
            Some(self.parse_condition()?)
        } else if self.check(&Token::When) {
            Some(self.parse_when_condition()?)
        } else {
            None
        };

        self.skip_newlines();

        // Parse effects and navigation (may be indented)
        let mut effects = Vec::new();
        let mut target = None;

        // Check for indent
        let in_indent = self.match_token(&Token::Indent);

        loop {
            self.skip_newlines();

            if in_indent && self.check(&Token::Dedent) {
                self.advance();
                break;
            }

            if self.check(&Token::EffectMarker) {
                effects.push(self.parse_effect()?);
            } else if self.check(&Token::Arrow) {
                target = Some(self.parse_navigation()?);
            } else if self.check(&Token::Newline) {
                self.advance();
            } else {
                break;
            }
        }

        // Handle non-indented navigation
        if target.is_none() && self.check(&Token::Arrow) {
            target = Some(self.parse_navigation()?);
        }

        let end_span = self.current_span();

        Ok(Choice {
            text,
            condition,
            effects,
            target,
            span: start_span.start..end_span.start,
        })
    }

    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::LBrace, "Expected {")?;

        let mut clauses = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            clauses.push(self.parse_condition_clause()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        let end_span = self.current_span();
        self.consume(&Token::RBrace, "Expected }")?;

        Ok(Condition {
            clauses,
            span: start_span.start..end_span.end,
        })
    }

    fn parse_condition_clause(&mut self) -> Result<ConditionClause, ParseError> {
        let start_span = self.current_span();

        // Check for negation
        let negated = self.match_token(&Token::Bang);

        // Expect identifier
        let first = match self.current_token() {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::InvalidCondition {
                    message: "Expected identifier".to_string(),
                    span: self.current_span(),
                })
            }
        };

        // Check for tag condition: source.tag
        if self.check(&Token::Dot) {
            self.advance();
            let tag = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidCondition {
                        message: "Expected tag name after .".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            let source = match first.as_str() {
                "crew" => TagSource::Crew,
                "ship" => TagSource::Ship,
                "cargo" => TagSource::Cargo,
                _ => {
                    return Err(ParseError::InvalidCondition {
                        message: format!(
                            "Invalid tag source: {}. Expected crew, ship, or cargo",
                            first
                        ),
                        span: start_span,
                    })
                }
            };

            return Ok(ConditionClause::Tag(TagCondition {
                source,
                tag,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Check for flag condition
        if first.as_str() == "flag" {
            let flag_name = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidCondition {
                        message: "Expected flag name".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            let (operator, value) = self.parse_optional_comparison()?;

            return Ok(ConditionClause::Flag(FlagCondition {
                flag: flag_name,
                negated,
                operator,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Check for resource condition
        if is_resource(&first) {
            let operator = self.parse_comparison_op()?;
            let value = self.parse_number()?;

            return Ok(ConditionClause::Resource(ResourceCondition {
                resource: first,
                operator,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Default to flag condition (identifier alone)
        Ok(ConditionClause::Flag(FlagCondition {
            flag: first,
            negated,
            operator: None,
            value: None,
            span: start_span.start..self.current_span().start,
        }))
    }

    fn parse_optional_comparison(&mut self) -> Result<(Option<CompareOp>, Option<FlagValue>), ParseError> {
        let op = match self.current_token() {
            Token::Ge => Some(CompareOp::Ge),
            Token::Le => Some(CompareOp::Le),
            Token::Gt => Some(CompareOp::Gt),
            Token::Lt => Some(CompareOp::Lt),
            Token::EqEq => Some(CompareOp::Eq),
            Token::Ne => Some(CompareOp::Ne),
            _ => return Ok((None, None)),
        };

        if op.is_some() {
            self.advance();
            let value = self.parse_flag_value()?;
            return Ok((op, Some(value)));
        }

        Ok((None, None))
    }

    fn parse_comparison_op(&mut self) -> Result<CompareOp, ParseError> {
        let op = match self.current_token() {
            Token::Ge => CompareOp::Ge,
            Token::Le => CompareOp::Le,
            Token::Gt => CompareOp::Gt,
            Token::Lt => CompareOp::Lt,
            Token::EqEq => CompareOp::Eq,
            Token::Ne => CompareOp::Ne,
            _ => {
                return Err(ParseError::InvalidCondition {
                    message: "Expected comparison operator".to_string(),
                    span: self.current_span(),
                })
            }
        };
        self.advance();
        Ok(op)
    }

    fn parse_number(&mut self) -> Result<i64, ParseError> {
        match self.current_token() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(n)
            }
            Token::Float(f) => {
                let n = *f as i64;
                self.advance();
                Ok(n)
            }
            _ => Err(ParseError::InvalidCondition {
                message: "Expected number".to_string(),
                span: self.current_span(),
            }),
        }
    }

    fn parse_flag_value(&mut self) -> Result<FlagValue, ParseError> {
        match self.current_token() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(FlagValue::Int(n))
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(FlagValue::String(s))
            }
            Token::Identifier(s) if s == "true" => {
                self.advance();
                Ok(FlagValue::Bool(true))
            }
            Token::Identifier(s) if s == "false" => {
                self.advance();
                Ok(FlagValue::Bool(false))
            }
            _ => Err(ParseError::InvalidCondition {
                message: "Expected value".to_string(),
                span: self.current_span(),
            }),
        }
    }

    fn parse_effect(&mut self) -> Result<Effect, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::EffectMarker, "Expected ~ for effect")?;

        let keyword = match self.current_token() {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::InvalidEffect {
                    message: "Expected effect keyword".to_string(),
                    span: self.current_span(),
                })
            }
        };

        // Resource effect
        if is_resource(&keyword) {
            let operator = self.parse_assign_op()?;
            let value = self.parse_number()?;
            self.skip_newlines();

            return Ok(Effect::Resource(ResourceEffect {
                resource: keyword,
                operator,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Flag effect
        if keyword.as_str() == "flag" {
            let flag_name = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidEffect {
                        message: "Expected flag name".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            let value = if self.match_token(&Token::Eq) {
                self.parse_flag_value()?
            } else {
                FlagValue::Bool(true)
            };

            self.skip_newlines();

            return Ok(Effect::Flag(FlagEffect {
                flag: flag_name,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // AddCard effect
        if keyword.as_str() == "addCard" {
            let card_id = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidEffect {
                        message: "Expected card ID".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            self.skip_newlines();

            return Ok(Effect::AddCard(AddCardEffect {
                card_id,
                span: start_span.start..self.current_span().start,
            }));
        }

        // RemoveCards effect
        if keyword.as_str() == "removeCards" {
            let mut pattern = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s.to_string()
                }
                _ => {
                    return Err(ParseError::InvalidEffect {
                        message: "Expected card pattern".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            // Handle wildcard pattern like cargo_*
            if self.check(&Token::ChoiceMarker) {
                self.advance();
                pattern.push('*');
            }

            self.skip_newlines();

            return Ok(Effect::RemoveCards(RemoveCardsEffect {
                pattern: SmolStr::new(&pattern),
                span: start_span.start..self.current_span().start,
            }));
        }

        // Chronicle effect
        if keyword.as_str() == "chronicle" {
            let title = match self.current_token() {
                Token::String(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidEffect {
                        message: "Expected chronicle title in quotes".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            self.skip_newlines();

            // Chronicle text may be on the next line (indented)
            let mut text_parts = Vec::new();
            let in_indent = self.match_token(&Token::Indent);

            if in_indent {
                while !self.check(&Token::Dedent) && !self.is_at_end() {
                    match self.current_token() {
                        Token::Identifier(s) | Token::Text(s) => {
                            text_parts.push(s.to_string());
                            self.advance();
                        }
                        Token::Number(n) => {
                            text_parts.push(n.to_string());
                            self.advance();
                        }
                        Token::String(s) => {
                            text_parts.push(s.to_string());
                            self.advance();
                        }
                        Token::Dot | Token::Comma | Token::Colon => {
                            let ch = match self.current_token() {
                                Token::Dot => ".",
                                Token::Comma => ",",
                                Token::Colon => ":",
                                _ => "",
                            };
                            text_parts.push(ch.to_string());
                            self.advance();
                        }
                        Token::Newline => {
                            self.advance();
                        }
                        _ => break,
                    }
                }
                if self.check(&Token::Dedent) {
                    self.advance();
                }
            }

            let text = SmolStr::new(&text_parts.join(" ").trim());

            return Ok(Effect::Chronicle(ChronicleEffect {
                title,
                text,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Damage effect
        if keyword.as_str() == "damage" {
            let target = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidEffect {
                        message: "Expected damage target".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            let value = self.parse_number()?;
            self.skip_newlines();

            return Ok(Effect::Damage(DamageEffect {
                target,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Reputation effect
        if keyword.as_str() == "reputation" {
            let faction = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidEffect {
                        message: "Expected faction name".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            let operator = self.parse_assign_op()?;
            let value = self.parse_number()?;
            self.skip_newlines();

            return Ok(Effect::Reputation(ReputationEffect {
                faction,
                operator,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Script effect: ~ script: <code> or ~ script: |
        if keyword.as_str() == "script" {
            // Expect colon after "script"
            self.consume(&Token::Colon, "Expected ':' after 'script'")?;

            // Check if this is a multi-line script block (indicated by being on next line after indent)
            // or an inline script
            let script_source = self.collect_script_source()?;

            return Ok(Effect::Script(ScriptEffect {
                source: SmolStr::new(&script_source),
                span: start_span.start..self.current_span().start,
            }));
        }

        Err(ParseError::InvalidEffect {
            message: format!("Unknown effect: {}", keyword),
            span: start_span,
        })
    }

    fn parse_assign_op(&mut self) -> Result<AssignOp, ParseError> {
        let op = match self.current_token() {
            Token::PlusEq => AssignOp::Add,
            Token::MinusEq => AssignOp::Sub,
            Token::Eq => AssignOp::Set,
            _ => {
                return Err(ParseError::InvalidEffect {
                    message: "Expected +=, -=, or =".to_string(),
                    span: self.current_span(),
                })
            }
        };
        self.advance();
        Ok(op)
    }

    fn parse_navigation(&mut self) -> Result<NavigationTarget, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::Arrow, "Expected ->")?;

        let target = match self.current_token() {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "passage name or END".to_string(),
                    found: format!("{}", self.current_token()),
                    span: self.current_span(),
                })
            }
        };

        let is_end = target.as_str() == "END";
        self.skip_newlines();

        Ok(NavigationTarget {
            target,
            is_end,
            span: start_span.start..self.current_span().start,
        })
    }

    /// Collect script source from `~ script:` effect.
    /// Handles both inline and multi-line (indented block) formats.
    fn collect_script_source(&mut self) -> Result<String, ParseError> {
        // Skip any whitespace after the colon
        // Check if we have an indent (multi-line block)
        if self.match_token(&Token::Newline) {
            if self.match_token(&Token::Indent) {
                // Multi-line script block
                let mut lines = Vec::new();
                let mut current_line = Vec::new();

                while !self.check(&Token::Dedent) && !self.is_at_end() {
                    match self.current_token() {
                        Token::Newline => {
                            if !current_line.is_empty() {
                                lines.push(current_line.join(" "));
                                current_line.clear();
                            }
                            self.advance();
                        }
                        _ => {
                            // Collect all tokens on this line
                            let part = match self.current_token() {
                                Token::Identifier(s) => s.to_string(),
                                Token::Number(n) => n.to_string(),
                                Token::Float(f) => f.to_string(),
                                Token::String(s) => format!("\"{}\"", s),
                                Token::LBrace => "{".to_string(),
                                Token::RBrace => "}".to_string(),
                                Token::LParen => "(".to_string(),
                                Token::RParen => ")".to_string(),
                                Token::LBracket => "[".to_string(),
                                Token::RBracket => "]".to_string(),
                                Token::Comma => ",".to_string(),
                                Token::Colon => ":".to_string(),
                                Token::Dot => ".".to_string(),
                                Token::Bang => "!".to_string(),
                                Token::Ge => ">=".to_string(),
                                Token::Le => "<=".to_string(),
                                Token::EqEq => "==".to_string(),
                                Token::Ne => "!=".to_string(),
                                Token::Gt => ">".to_string(),
                                Token::Lt => "<".to_string(),
                                Token::PlusEq => "+=".to_string(),
                                Token::MinusEq => "-=".to_string(),
                                Token::Eq => "=".to_string(),
                                Token::When => "when".to_string(),
                                _ => String::new(),
                            };
                            if !part.is_empty() {
                                current_line.push(part);
                            }
                            self.advance();
                        }
                    }
                }

                if !current_line.is_empty() {
                    lines.push(current_line.join(" "));
                }

                if self.check(&Token::Dedent) {
                    self.advance();
                }

                return Ok(lines.join("\n"));
            }
        }

        // Inline script - collect rest of line
        let mut parts = Vec::new();
        while !self.check(&Token::Newline) && !self.is_at_end() {
            let part = match self.current_token() {
                Token::Identifier(s) => s.to_string(),
                Token::Number(n) => n.to_string(),
                Token::Float(f) => f.to_string(),
                Token::String(s) => format!("\"{}\"", s),
                Token::LBrace => "{".to_string(),
                Token::RBrace => "}".to_string(),
                Token::LParen => "(".to_string(),
                Token::RParen => ")".to_string(),
                Token::LBracket => "[".to_string(),
                Token::RBracket => "]".to_string(),
                Token::Comma => ",".to_string(),
                Token::Colon => ":".to_string(),
                Token::Dot => ".".to_string(),
                Token::Bang => "!".to_string(),
                Token::Ge => ">=".to_string(),
                Token::Le => "<=".to_string(),
                Token::EqEq => "==".to_string(),
                Token::Ne => "!=".to_string(),
                Token::Gt => ">".to_string(),
                Token::Lt => "<".to_string(),
                Token::PlusEq => "+=".to_string(),
                Token::MinusEq => "-=".to_string(),
                Token::Eq => "=".to_string(),
                Token::When => "when".to_string(),
                _ => String::new(),
            };
            if !part.is_empty() {
                parts.push(part);
            }
            self.advance();
        }

        self.skip_newlines();
        Ok(parts.join(" "))
    }

    /// Parse `when <condition>` syntax for choice conditions.
    /// This is an alternative to `{ condition }` that reads until end of line.
    fn parse_when_condition(&mut self) -> Result<Condition, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::When, "Expected 'when'")?;

        // Parse condition clauses until newline
        let mut clauses = Vec::new();

        while !self.check(&Token::Newline) && !self.is_at_end() {
            clauses.push(self.parse_condition_clause()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(Condition {
            clauses,
            span: start_span.start..self.current_span().start,
        })
    }

    /// Parse an inline Rhai block: `{ ... }`.
    /// Handles nested braces by counting depth.
    fn parse_rhai_block(&mut self) -> Result<RhaiBlock, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::LBrace, "Expected '{'")?;

        // Track source position in original source
        let source_start = self.current_span().start;
        let mut depth = 1;

        // Advance through tokens, tracking brace depth
        while depth > 0 && !self.is_at_end() {
            match self.current_token() {
                Token::LBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RBrace => {
                    depth -= 1;
                    if depth > 0 {
                        self.advance();
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        let source_end = self.current_span().start;

        if depth != 0 {
            return Err(ParseError::UnexpectedToken {
                expected: "closing '}'".to_string(),
                found: "end of file".to_string(),
                span: self.current_span(),
            });
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        // Extract the Rhai source from the original source string
        let rhai_source = self.source[source_start..source_end].trim();

        Ok(RhaiBlock {
            source: SmolStr::new(rhai_source),
            span: start_span.start..self.current_span().start,
        })
    }
}

/// Parse a scene file from source code
pub fn parse(source: &str, filename: &str) -> Result<SceneFile, ParseError> {
    let mut parser = Parser::new(source, filename);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_scene() {
        let source = r#"---
id: test_scene
title: Test Scene
tags: [test]
context: journey
weight: 10
cooldown: 5
---

=== intro

This is a test.

* [First choice]
  ~ credits += 10
  -> END

* [Second choice]
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        assert_eq!(scene.frontmatter.id.as_str(), "test_scene");
        assert_eq!(scene.frontmatter.title.as_str(), "Test Scene");
        assert_eq!(scene.passages.len(), 1);
        assert_eq!(scene.passages[0].name.as_str(), "intro");
    }

    #[test]
    fn test_parse_choice_with_condition() {
        let source = r#"---
id: cond_test
title: Condition Test
tags: []
context: port
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Tagged choice] { crew.engineering }
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert!(choice.condition.is_some());
            let condition = choice.condition.as_ref().unwrap();
            assert_eq!(condition.clauses.len(), 1);

            if let ConditionClause::Tag(tag_cond) = &condition.clauses[0] {
                assert_eq!(tag_cond.source, TagSource::Crew);
                assert_eq!(tag_cond.tag.as_str(), "engineering");
            } else {
                panic!("Expected tag condition");
            }
        } else {
            panic!("Expected choice");
        }
    }

    #[test]
    fn test_parse_multiple_effects() {
        let source = r#"---
id: effect_test
title: Effect Test
tags: []
context: journey
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Do something]
  ~ damage hull 10
  ~ flag test_flag
  ~ credits += 50
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert_eq!(choice.effects.len(), 3);

            assert!(matches!(&choice.effects[0], Effect::Damage(_)));
            assert!(matches!(&choice.effects[1], Effect::Flag(_)));
            assert!(matches!(&choice.effects[2], Effect::Resource(_)));
        } else {
            panic!("Expected choice");
        }
    }

    #[test]
    fn test_parse_branching_scene() {
        let source = r#"---
id: branching
title: Branching Scene
tags: []
context: port
weight: 10
cooldown: 5
---

=== intro

Start here.

* [Go left]
  -> left

* [Go right]
  -> right

=== left

You went left.

* [Done]
  -> END

=== right

You went right.

* [Done]
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        assert_eq!(scene.passages.len(), 3);
        assert_eq!(scene.passages[0].name.as_str(), "intro");
        assert_eq!(scene.passages[1].name.as_str(), "left");
        assert_eq!(scene.passages[2].name.as_str(), "right");
    }

    #[test]
    fn test_parse_when_condition() {
        let source = r#"---
id: when_test
title: When Test
tags: []
context: journey
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Choice with when] when credits >= 100
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert!(choice.condition.is_some());
            let condition = choice.condition.as_ref().unwrap();
            assert_eq!(condition.clauses.len(), 1);

            if let ConditionClause::Resource(res_cond) = &condition.clauses[0] {
                assert_eq!(res_cond.resource.as_str(), "credits");
                assert_eq!(res_cond.operator, CompareOp::Ge);
                assert_eq!(res_cond.value, 100);
            } else {
                panic!("Expected resource condition");
            }
        } else {
            panic!("Expected choice");
        }
    }

    #[test]
    fn test_parse_script_effect() {
        let source = r#"---
id: script_test
title: Script Test
tags: []
context: journey
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Do something]
  ~ script: damage("hull", 10);
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert_eq!(choice.effects.len(), 1);

            if let Effect::Script(script) = &choice.effects[0] {
                assert!(script.source.contains("damage"));
            } else {
                panic!("Expected script effect");
            }
        } else {
            panic!("Expected choice");
        }
    }

    #[test]
    fn test_parse_inline_rhai_block() {
        let source = r#"---
id: rhai_block_test
title: Rhai Block Test
tags: []
context: journey
weight: 10
cooldown: 5
---

=== intro

{
  if rng_float() < 0.5 {
    damage("hull", 5);
  }
}

Some text here.

* [Continue]
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        // Should have RhaiBlock, Prose, Choice in content
        let has_rhai_block = passage.content.iter().any(|c| matches!(c, PassageContent::RhaiBlock(_)));
        assert!(has_rhai_block, "Expected RhaiBlock in passage content");
    }
}
