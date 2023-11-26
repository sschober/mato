---
title: Refactoring diary: warp_at
author: Sven Schober
---

# Motivation

Currently, the function `wrap_at()` is part of the `markdown` `
Renderer`.


```
fn wrap_at(&mut self, s: String, col: usize) -> String {
    let mut result = String::new();
    if s.starts_with(' ') {
        result = " ".to_string();
        self.char_index += 1;
    }
    for word in s.split(&[' ', '\n']) {
        if self.char_index + word.len() < col {
            if !result.is_empty() && result != " " {
                result = format!("{} {}", result, word);
            } else {
                result = format!("{}{}", result, word);
            }
            self.char_index += word.len();
        } else {
            result = format!("{}\n{}", result, word);
            self.char_index = word.len();
        }
        self.char_index += 1;
    }
    result
}
```
