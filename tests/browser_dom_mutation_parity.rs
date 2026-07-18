use tetherscript::browser_agent::BrowserPage;

#[test]
fn advanced_dom_mutations_preserve_browser_shapes() {
    let html = "<main id='app'><p id='a'>A</p><p id='b'>B</p></main>";
    let script = "let app=document.getElementById('app');\
        let a=document.getElementById('a');let b=document.getElementById('b');\
        let frag=document.createDocumentFragment();let em=document.createElement('em');\
        em.textContent='E';frag.appendChild(em);app.insertBefore(frag,b);\
        let clone=a.cloneNode(true);clone.id='c';let replaced=app.replaceChild(clone,b);\
        let imported=document.importNode(em,true);let adopted=document.adoptNode(a);\
        [app.textContent,app.children.length,replaced.id,imported.textContent,\
        adopted.id,adopted.isConnected].join('|');";
    let mut page = BrowserPage::from_html("mem://dom-mutation", html);
    let value = page
        .eval_js(script)
        .expect("DOM mutation script should run");

    assert_eq!(value.display(), "EA|2|b|E|a|false");
}
