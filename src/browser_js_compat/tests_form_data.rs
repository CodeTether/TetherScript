use super::super::*;

#[test]
fn form_data_collects_forms_and_mutates_entries() {
    let result = eval_with_dom(
        "<form id='f'><input name='q' value='one'><input name='skip' disabled value='x'><input type='checkbox' name='ok' checked></form>",
        "let d=new FormData(document.getElementById('f')); \
         d.append('q','two'); d.append('upload',File(['x'],'x.txt'),'sent.txt'); \
         let all=d.getAll('q'); let rows=d.entries(); let file=d.get('upload'); \
         let before=d.has('skip'); d.delete('ok'); \
         d.get('q')+'|'+all.length+'|'+all[1]+'|'+before+'|'+d.has('ok')+'|'+rows.length+'|'+file.name+'|'+file.size;",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("one|2|two|false|false|4|sent.txt|1".into())
    );
}

#[test]
fn form_data_set_iterators_and_for_each_are_deterministic() {
    let result = eval_with_dom(
        "<main></main>",
        "let d=new FormData(); \
         d.append('a','1'); d.append('a','2'); d.append('b','3'); \
         d.set('a','9'); d.set('c','4'); \
         let rows=d.entries(); let seen=''; \
         d.forEach(function(value,name,self){ seen=seen+name+'='+value+':' + (self===d) + ';'; }); \
         d.keys().join(',')+'|'+d.values().join(',')+'|'+rows.length+'|'+rows[0][0]+'='+rows[0][1]+'|'+d.getAll('a').length+'|'+seen;",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("a,b,c|9,3,4|3|a=9|1|a=9:true;b=3:true;c=4:true;".into())
    );
}
