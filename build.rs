fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf files from the proto/ directory.
    prost_build::compile_protos(
        &[
            "proto/common.proto",
            "proto/king.proto",
            "proto/queen.proto",
            "proto/knight.proto",
            "proto/bishop.proto",
            "proto/pawn.proto",
            "proto/pieces.proto",
            "proto/board.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
