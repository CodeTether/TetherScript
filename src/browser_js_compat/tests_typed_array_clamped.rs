use super::super::*;

#[test]
fn uint8_clamped_array_rounds_constructor_and_index_assignments() {
    let result = eval_with_dom(
        "<main></main>",
        "let a=new Uint8ClampedArray([-1,0.5,1.5,2.5,254.6,300]);\
         a[0]=12.5;a[1]=999;a.set([300],2);a[99]=7;\
         a.join(',')+':'+Uint8ClampedArray.BYTES_PER_ELEMENT+':'+\
         (a instanceof Uint8ClampedArray)+':'+a.length+':'+(a[99]===undefined);",
    )
    .unwrap();

    assert_eq!(
        result.value.display(),
        "12,255,255,2,255,255:1:true:6:true"
    );
}
