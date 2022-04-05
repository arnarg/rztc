extern crate cc;
extern crate bindgen;

use std::path::PathBuf;
use std::env;
use std::vec;
use bindgen::builder;

fn main() {
    // Default ZeroTierCore cpp files
    let mut files = vec![
        "zerotierone/node/AES.cpp",
        "zerotierone/node/AES_aesni.cpp",
        "zerotierone/node/AES_armcrypto.cpp",
        "zerotierone/node/C25519.cpp",
        "zerotierone/node/Capability.cpp",
        "zerotierone/node/CertificateOfMembership.cpp",
        "zerotierone/node/CertificateOfOwnership.cpp",
        "zerotierone/node/Identity.cpp",
        "zerotierone/node/IncomingPacket.cpp",
        "zerotierone/node/InetAddress.cpp",
        "zerotierone/node/Membership.cpp",
        "zerotierone/node/Multicaster.cpp",
        "zerotierone/node/Network.cpp",
        "zerotierone/node/NetworkConfig.cpp",
        "zerotierone/node/Node.cpp",
        "zerotierone/node/OutboundMulticast.cpp",
        "zerotierone/node/Packet.cpp",
        "zerotierone/node/Path.cpp",
        "zerotierone/node/Peer.cpp",
        "zerotierone/node/Poly1305.cpp",
        "zerotierone/node/Revocation.cpp",
        "zerotierone/node/Salsa20.cpp",
        "zerotierone/node/SelfAwareness.cpp",
        "zerotierone/node/SHA512.cpp",
        "zerotierone/node/Switch.cpp",
        "zerotierone/node/Tag.cpp",
        "zerotierone/node/Topology.cpp",
        "zerotierone/node/Trace.cpp",
        "zerotierone/node/Utils.cpp",
        "zerotierone/node/Bond.cpp",
    ];

    let target = env::var("TARGET").unwrap();
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut cfg = cc::Build::new();

    // x86_64 specific config
    if target.contains("x86_64") {
        files.append(&mut vec![
            "zerotierone/ext/x64-salsa2012-asm/salsa2012.s"
        ]);
        if target.contains("linux") {
            files.append(&mut vec![
                "zerotierone/ext/ed25519-amd64-asm/choose_t.s",
                "zerotierone/ext/ed25519-amd64-asm/consts.s",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_add.s",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_freeze.s",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_mul.s",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_square.s",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_sub.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_add_p1p1.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_dbl_p1p1.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_nielsadd2.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_nielsadd_p1p1.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_p1p1_to_p2.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_p1p1_to_p3.s",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_pnielsadd_p1p1.s",
                "zerotierone/ext/ed25519-amd64-asm/heap_rootreplaced.s",
                "zerotierone/ext/ed25519-amd64-asm/heap_rootreplaced_1limb.s",
                "zerotierone/ext/ed25519-amd64-asm/heap_rootreplaced_2limbs.s",
                "zerotierone/ext/ed25519-amd64-asm/heap_rootreplaced_3limbs.s",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_add.s",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_barrett.s",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_lt.s",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_sub_nored.s",
                "zerotierone/ext/ed25519-amd64-asm/ull4_mul.s",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_getparity.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_invert.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_iseq.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_iszero.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_neg.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_pack.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_pow2523.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_setint.c",
                "zerotierone/ext/ed25519-amd64-asm/fe25519_unpack.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_add.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_base.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_double.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_double_scalarmult.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_isneutral.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_multi_scalarmult.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_pack.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_scalarmult_base.c",
                "zerotierone/ext/ed25519-amd64-asm/ge25519_unpackneg.c",
                "zerotierone/ext/ed25519-amd64-asm/hram.c",
                "zerotierone/ext/ed25519-amd64-asm/index_heap.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_from32bytes.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_from64bytes.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_from_shortsc.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_iszero.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_mul.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_mul_shortsc.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_slide.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_to32bytes.c",
                "zerotierone/ext/ed25519-amd64-asm/sc25519_window4.c",
                "zerotierone/ext/ed25519-amd64-asm/sign.c"
            ]);
        }
    }
    if target.contains("aarch64") {
        files.append(&mut vec![
            "ext/arm32-neon-salsa2012-asm/salsa2012.s"
        ]);
    }

    cfg.files(files)
        .cpp(true)
        .include("zerotierone/include")
        .cpp_link_stdlib("stdc++")
        .warnings(false)
        .pic(true)
        .opt_level(3)
        .flag("-fstack-protector")
        .compile("libzerotierone.a");

    let bindings = builder()
        .header("zerotierone/include/ZeroTierOne.h")
        .allowlist_type("ZT_.*")
        .allowlist_function("ZT_.*")
        .allowlist_var("ZT_.*")
        .clang_arg("--includestdbool.h")
        .generate()
        .expect("unable to generate bindings for zerotiercore");

    bindings.write_to_file(dst.join("bindings.rs"))
         .expect("unable to write bindings to file");
}
