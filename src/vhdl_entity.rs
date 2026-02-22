use serde::Serialize;
use std::collections::HashMap;

use crate::vhdl_constant::{VhdlConstant, parse_constants};
use crate::vhdl_generic::{VhdlGeneric, parse_generics};
use crate::vhdl_port::{VhdlPort, parse_ports};
use crate::vhdl_range::VhdlRange;
use crate::vhdl_signal::{VhdlSignal, parse_signals};
use crate::vhdl_token::{TokenType, VhdlToken};
use crate::vhdl_type::VhdlType;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlArchitecture {
    pub name: String,
    pub constants: Vec<VhdlConstant>,
    pub signals: Vec<VhdlSignal>,
    pub children: Vec<VhdlEntity>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct VhdlEntity {
    pub name: String,
    pub generics: Vec<VhdlGeneric>,
    pub ports: Vec<VhdlPort>,
    pub architectures: Vec<VhdlArchitecture>,
    pub file_name: String,
    pub line: usize,
}

impl VhdlEntity {
    /// Parse a token stream into VHDL entities, appending to the given entities vec.
    pub fn parse_entities(entities: &mut Vec<VhdlEntity>, tokens: &[VhdlToken], file_name: &str) {
        let mut idx = 0;
        let mut current_entity: Option<VhdlEntity> = None;
        let mut current_arch_entity_idx: Option<usize> = None;
        let mut current_arch_name: Option<String> = None;
        let mut generate_stack: Vec<usize> = Vec::new();

        while idx < tokens.len() {
            let t = &tokens[idx];

            match &t.token_type {
                // ── Entity declaration ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("entity") => {
                    idx += 1;
                    if let Some(name_token) = tokens.get(idx)
                        && let TokenType::Identifier(name) = &name_token.token_type
                    {
                        idx += 1;
                        if let Some(is_token) = tokens.get(idx)
                            && let TokenType::Identifier(is_id) = &is_token.token_type
                            && is_id.eq_ignore_ascii_case("is")
                        {
                            current_entity = Some(VhdlEntity {
                                name: name.clone(),
                                generics: Vec::new(),
                                ports: Vec::new(),
                                architectures: Vec::new(),
                                file_name: file_name.to_string(),
                                line: t.line,
                            });
                        }
                    }
                }

                // ── End entity / architecture ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("end") => {
                    idx += 1;
                    if let Some(next_token) = tokens.get(idx) {
                        if let TokenType::Identifier(next_id) = &next_token.token_type {
                            if next_id.eq_ignore_ascii_case("entity")
                                || (current_entity.is_some()
                                    && next_id.eq_ignore_ascii_case(
                                        &current_entity.as_ref().unwrap().name,
                                    ))
                            {
                                if let Some(ent) = current_entity.take() {
                                    entities.push(ent);
                                }
                                idx += 1;
                            } else if next_id.eq_ignore_ascii_case("architecture")
                                || next_id.eq_ignore_ascii_case("behavioral")
                                || next_id.eq_ignore_ascii_case("structural")
                                || next_id.eq_ignore_ascii_case("rtl")
                                || (current_arch_name.is_some()
                                    && next_id
                                        .eq_ignore_ascii_case(current_arch_name.as_ref().unwrap()))
                            {
                                current_arch_entity_idx = None;
                                current_arch_name = None;
                                generate_stack.clear();
                                idx += 1;
                            } else if next_id.eq_ignore_ascii_case("generate") {
                                let _ = generate_stack.pop();
                                idx += 1;
                            }
                        } else if let TokenType::Symbol(sym) = &next_token.token_type
                            && sym == ";"
                            && let Some(ent) = current_entity.take()
                        {
                            entities.push(ent);
                        }
                    }
                }

                // ── Generic declaration ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("generic") => {
                    if current_entity.is_some() && current_arch_entity_idx.is_none() {
                        let (generics, new_idx) = parse_generics(tokens, idx);
                        idx = new_idx;
                        if let Some(ent) = &mut current_entity {
                            ent.generics = generics;
                        }
                        continue;
                    }
                    // Inside architecture: skip generic blocks (e.g., component generics)
                    idx += 1;
                    if let Some(paren_token) = tokens.get(idx)
                        && let TokenType::Symbol(sym) = &paren_token.token_type
                        && sym == "("
                    {
                        idx += 1;
                        let mut paren_depth = 1;
                        while idx < tokens.len() && paren_depth > 0 {
                            if let TokenType::Symbol(s) = &tokens[idx].token_type {
                                if s == "(" {
                                    paren_depth += 1;
                                } else if s == ")" {
                                    paren_depth -= 1;
                                }
                            }
                            idx += 1;
                        }
                    }
                    continue;
                }

                // ── Port declaration ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("port") => {
                    if current_entity.is_some() && current_arch_entity_idx.is_none() {
                        let (ports, new_idx) = parse_ports(tokens, idx);
                        idx = new_idx;
                        if let Some(ent) = &mut current_entity {
                            ent.ports = ports;
                        }
                        continue;
                    }
                }

                // ── Architecture ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("architecture") => {
                    idx += 1;
                    if let Some(name_token) = tokens.get(idx)
                        && let TokenType::Identifier(arch_name) = &name_token.token_type
                    {
                        let arch_name_clone = arch_name.clone();
                        current_arch_name = Some(arch_name_clone.clone());
                        idx += 1;
                        if let Some(of_token) = tokens.get(idx)
                            && let TokenType::Identifier(of_id) = &of_token.token_type
                            && of_id.eq_ignore_ascii_case("of")
                        {
                            idx += 1;
                            if let Some(target_token) = tokens.get(idx)
                                && let TokenType::Identifier(target_name) = &target_token.token_type
                            {
                                current_arch_entity_idx = entities
                                    .iter()
                                    .position(|e| e.name.eq_ignore_ascii_case(target_name));

                                if let Some(arch_idx) = current_arch_entity_idx {
                                    entities[arch_idx].architectures.push(VhdlArchitecture {
                                        name: arch_name_clone,
                                        constants: Vec::new(),
                                        signals: Vec::new(),
                                        children: Vec::new(),
                                    });
                                }
                            }
                        }
                    }
                }

                // ── Signal inside architecture ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("signal") => {
                    if let Some(arch_idx) = current_arch_entity_idx {
                        let (signals, new_idx) = parse_signals(tokens, idx);
                        idx = new_idx;
                        if let Some(arch) = entities[arch_idx].architectures.last_mut() {
                            arch.signals.extend(signals);
                        }
                        continue;
                    }
                }

                // ── Constant inside architecture ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("constant") => {
                    if let Some(arch_idx) = current_arch_entity_idx {
                        let (constants, new_idx) = parse_constants(tokens, idx);
                        idx = new_idx;
                        if let Some(arch) = entities[arch_idx].architectures.last_mut() {
                            arch.constants.extend(constants);
                        }
                        continue;
                    }
                }

                // ── For-generate ──
                TokenType::Identifier(id) if id.eq_ignore_ascii_case("for") => {
                    if current_arch_entity_idx.is_some() {
                        idx += 2;
                        if let Some(in_t) = tokens.get(idx)
                            && let TokenType::Identifier(in_id) = &in_t.token_type
                            && in_id.eq_ignore_ascii_case("in")
                        {
                            idx += 1;
                            let mut range_str = String::new();
                            while idx < tokens.len() {
                                if let TokenType::Identifier(gen_id) = &tokens[idx].token_type
                                    && gen_id.eq_ignore_ascii_case("generate")
                                {
                                    let count = VhdlRange::parse_range_size_simple(&range_str);
                                    generate_stack.push(count);
                                    break;
                                }
                                if let TokenType::Identifier(i)
                                | TokenType::Number(i)
                                | TokenType::Symbol(i)
                                | TokenType::StringLiteral(i) = &tokens[idx].token_type
                                {
                                    if !range_str.is_empty() {
                                        range_str.push(' ');
                                    }
                                    range_str.push_str(i);
                                }
                                idx += 1;
                            }
                        }
                    }
                }

                // ── Component instantiation ──
                TokenType::Symbol(sym) if sym == ":" => {
                    if let Some(arch_idx) = current_arch_entity_idx {
                        let start_idx = idx;
                        let inst_line = t.line;

                        idx += 1;
                        let mut target_name = String::new();
                        if let Some(next_t) = tokens.get(idx)
                            && let TokenType::Identifier(nid) = &next_t.token_type
                        {
                            if nid.eq_ignore_ascii_case("entity") {
                                idx += 1;
                                if let Some(tgt_t) = tokens.get(idx)
                                    && let TokenType::Identifier(t_id) = &tgt_t.token_type
                                {
                                    let mut t_name = t_id.clone();
                                    if t_name.to_lowercase().starts_with("work.") {
                                        t_name = t_name[5..].to_string();
                                    }
                                    target_name = t_name;
                                }
                            } else {
                                target_name = nid.clone();
                                idx += 1;
                                if let Some(follow_t) = tokens.get(idx) {
                                    if let TokenType::Identifier(fid) = &follow_t.token_type {
                                        if !fid.eq_ignore_ascii_case("generic")
                                            && !fid.eq_ignore_ascii_case("port")
                                        {
                                            target_name.clear();
                                        }
                                    } else {
                                        target_name.clear();
                                    }
                                } else {
                                    target_name.clear();
                                }
                            }
                        }

                        if !target_name.is_empty() {
                            let mut generic_actuals: HashMap<String, String> = HashMap::new();
                            let mut local_idx = idx;
                            let mut inside_generic_map = false;
                            let mut inside_port_map = false;

                            while local_idx < tokens.len() {
                                match &tokens[local_idx].token_type {
                                    TokenType::Identifier(id)
                                        if (id.eq_ignore_ascii_case("port")
                                            || id.eq_ignore_ascii_case("generic")) =>
                                    {
                                        if let Some(next_t) = tokens.get(local_idx + 1)
                                            && let TokenType::Identifier(mid) = &next_t.token_type
                                            && mid.eq_ignore_ascii_case("map")
                                        {
                                            inside_generic_map = id.eq_ignore_ascii_case("generic");
                                            inside_port_map = id.eq_ignore_ascii_case("port");
                                            local_idx += 2;
                                            if let Some(TokenType::Symbol(s)) =
                                                tokens.get(local_idx).map(|t| &t.token_type)
                                                && s == "("
                                            {
                                                local_idx += 1;
                                            }
                                            continue;
                                        }
                                    }
                                    TokenType::Symbol(s) if s == ";" => {
                                        break;
                                    }
                                    TokenType::Symbol(s)
                                        if s == "=>" && (inside_generic_map || inside_port_map) =>
                                    {
                                        let mut key = String::new();
                                        let mut rev_idx = local_idx;
                                        let mut rev_paren_depth = 0;
                                        while rev_idx > 0 {
                                            rev_idx -= 1;
                                            match &tokens[rev_idx].token_type {
                                                TokenType::Symbol(sym) if sym == ")" => {
                                                    rev_paren_depth += 1;
                                                    key.insert_str(0, sym);
                                                }
                                                TokenType::Symbol(sym) if sym == "(" => {
                                                    if rev_paren_depth > 0 {
                                                        rev_paren_depth -= 1;
                                                        key.insert_str(0, sym);
                                                    } else {
                                                        break;
                                                    }
                                                }
                                                TokenType::Symbol(sym) if sym == "," => {
                                                    if rev_paren_depth == 0 {
                                                        break;
                                                    } else {
                                                        key.insert_str(0, sym);
                                                    }
                                                }
                                                TokenType::Identifier(i)
                                                | TokenType::Number(i)
                                                | TokenType::StringLiteral(i)
                                                | TokenType::Symbol(i) => {
                                                    key.insert_str(0, i);
                                                }
                                                _ => {}
                                            }
                                        }
                                        key = key.trim().to_string();

                                        let mut value = String::new();
                                        let mut search_idx = local_idx + 1;
                                        let mut paren_depth = 0;
                                        while search_idx < tokens.len() {
                                            match &tokens[search_idx].token_type {
                                                TokenType::Symbol(sym) if sym == "(" => {
                                                    paren_depth += 1;
                                                    value.push_str(sym);
                                                }
                                                TokenType::Symbol(sym) if sym == ")" => {
                                                    if paren_depth == 0 {
                                                        break;
                                                    } else {
                                                        paren_depth -= 1;
                                                        value.push_str(sym);
                                                    }
                                                }
                                                TokenType::Symbol(sym)
                                                    if sym == "," && paren_depth == 0 =>
                                                {
                                                    break;
                                                }
                                                TokenType::Identifier(i)
                                                | TokenType::Number(i)
                                                | TokenType::StringLiteral(i)
                                                | TokenType::Symbol(i) => {
                                                    if !value.is_empty()
                                                        && !matches!(
                                                            &tokens[search_idx].token_type,
                                                            TokenType::Symbol(_)
                                                        )
                                                    {
                                                        value.push(' ');
                                                    }
                                                    value.push_str(i);
                                                }
                                                _ => {}
                                            }
                                            search_idx += 1;
                                        }

                                        if inside_generic_map && !key.is_empty() {
                                            generic_actuals.insert(key, value.trim().to_string());
                                        }

                                        local_idx = search_idx;
                                        continue;
                                    }
                                    _ => {}
                                }
                                local_idx += 1;
                            }

                            idx = local_idx;

                            let mut multiplier: usize = generate_stack.iter().product();
                            if multiplier == 0 {
                                multiplier = 1;
                            }

                            let mut child_generics = Vec::new();
                            for (k, v) in &generic_actuals {
                                child_generics.push(VhdlGeneric {
                                    name: k.clone(),
                                    vhdl_type: VhdlType {
                                        name: String::new(),
                                        range: None,
                                    },
                                    default_value: None,
                                    actual_value: Some(v.clone()),
                                });
                            }

                            for _ in 0..multiplier {
                                if let Some(arch) = entities[arch_idx].architectures.last_mut() {
                                    arch.children.push(VhdlEntity {
                                        name: target_name.clone(),
                                        generics: child_generics.clone(),
                                        ports: Vec::new(),
                                        architectures: Vec::new(),
                                        file_name: file_name.to_string(),
                                        line: inst_line,
                                    });
                                }
                            }
                        } else {
                            idx = start_idx;
                        }
                    }
                }

                _ => {}
            }
            idx += 1;
        }
    }
}
