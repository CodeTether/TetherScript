use tetherscript::browser_agent::BrowserPage;

pub(super) fn object_shape() {
    let mut page = BrowserPage::from_html("mem://match-media-shape", "<main>V</main>");
    let value = page
        .eval_js("let m=window.matchMedia('(min-width: 600px)'); typeof m + ':' + m.media + ':' + m.matches + ':' + typeof m.addEventListener + ':' + typeof m.removeEventListener")
        .unwrap();

    assert_eq!(
        value.display(),
        "object:(min-width: 600px):false:function:function"
    );
}

pub(super) fn listeners_and_removal() {
    let mut page = BrowserPage::from_html("mem://match-media-change", "<main>V</main>");
    let value = page
        .eval_js("let m=matchMedia('(min-width: 600px)');let out='';let keep=function(e){out+='K'+e.matches+';';};let gone=function(){out+='G';};m.addEventListener('change',keep);m.addEventListener('change',gone);m.removeEventListener('change',gone);m.dispatchEvent({type:'change'});out")
        .unwrap();

    assert_eq!(value.display(), "Kfalse;");
}
