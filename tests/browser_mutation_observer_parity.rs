use tetherscript::browser_js::eval_with_dom;

#[test]
fn mutation_observer_delivers_attribute_and_child_records() {
    let script = "let root=document.getElementById('root');\
        let observer=MutationObserver(function(r){console.log(r.length+'|'\
        +r[0].type+':'+r[0].attributeName+':'+r[0].oldValue+'|'\
        +r[1].type+':'+r[1].attributeName+':'+r[1].oldValue+'|'\
        +r[2].type+':'+r[2].addedNodes.length+':'+r[2].removedNodes.length);});\
        observer.observe(root,{attributes:true,attributeOldValue:true,childList:true});\
        root.setAttribute('data-x','one');root.setAttribute('data-x','two');\
        root.appendChild(document.createElement('span'));'sync';";
    let result = eval_with_dom("<main id='root'></main>", script).unwrap();

    assert_eq!(result.value.display(), "sync");
    assert_eq!(
        result.console,
        ["3|attributes:data-x:null|attributes:data-x:one|childList:1:0"]
    );
}
