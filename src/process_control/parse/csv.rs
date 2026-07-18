pub(super) fn parse(line: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    for character in line.chars() {
        match character {
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(character),
        }
    }
    if quoted {
        return Err(format!("process_list: malformed CSV row `{line}`"));
    }
    fields.push(field);
    Ok(fields)
}
