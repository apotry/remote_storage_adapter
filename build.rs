use protobuf_codegen_pure::Customize;

fn main() {
    println!("cargo:rerun-if-changed=\"placehoder_to_trigger_build\"");

    let generated_with_pure_dir = "src/prompb";

    protobuf_codegen_pure::Codegen::new()
        .customize(Customize {
            gen_mod_rs: Some(true),
            //serde_derive: Some(true),
            ..Default::default()
        })
        .out_dir(generated_with_pure_dir)
        .input("src/prompb/remote.proto")
        .input("src/prompb/types.proto")
        .include("src/prompb")
        .include("src/prompb/gogoproto")
        .run()
        .expect("protoc");
}
