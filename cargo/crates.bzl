"""
@generated
cargo-raze generated Bazel file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")  # buildifier: disable=load

def raze_fetch_remote_crates():
    """This function defines a collection of repos and should be called in a WORKSPACE file"""
    maybe(
        http_archive,
        name = "raze__aho_corasick__0_7_18",
        url = "https://crates.io/api/v1/crates/aho-corasick/0.7.18/download",
        type = "tar.gz",
        sha256 = "1e37cfd5e7657ada45f742d6e99ca5788580b5c529dc78faf11ece6dc702656f",
        strip_prefix = "aho-corasick-0.7.18",
        build_file = Label("//cargo/remote:BUILD.aho-corasick-0.7.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__anyhow__1_0_52",
        url = "https://crates.io/api/v1/crates/anyhow/1.0.52/download",
        type = "tar.gz",
        sha256 = "84450d0b4a8bd1ba4144ce8ce718fbc5d071358b1e5384bace6536b3d1f2d5b3",
        strip_prefix = "anyhow-1.0.52",
        build_file = Label("//cargo/remote:BUILD.anyhow-1.0.52.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__arrayvec__0_5_2",
        url = "https://crates.io/api/v1/crates/arrayvec/0.5.2/download",
        type = "tar.gz",
        sha256 = "23b62fc65de8e4e7f52534fb52b0f3ed04746ae267519eef2a83941e8085068b",
        strip_prefix = "arrayvec-0.5.2",
        build_file = Label("//cargo/remote:BUILD.arrayvec-0.5.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__assert_cmd__1_0_8",
        url = "https://crates.io/api/v1/crates/assert_cmd/1.0.8/download",
        type = "tar.gz",
        sha256 = "c98233c6673d8601ab23e77eb38f999c51100d46c5703b17288c57fddf3a1ffe",
        strip_prefix = "assert_cmd-1.0.8",
        build_file = Label("//cargo/remote:BUILD.assert_cmd-1.0.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__atty__0_2_14",
        url = "https://crates.io/api/v1/crates/atty/0.2.14/download",
        type = "tar.gz",
        sha256 = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8",
        strip_prefix = "atty-0.2.14",
        build_file = Label("//cargo/remote:BUILD.atty-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__autocfg__1_0_1",
        url = "https://crates.io/api/v1/crates/autocfg/1.0.1/download",
        type = "tar.gz",
        sha256 = "cdb031dd78e28731d87d56cc8ffef4a8f36ca26c38fe2de700543e627f8a464a",
        strip_prefix = "autocfg-1.0.1",
        build_file = Label("//cargo/remote:BUILD.autocfg-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bit_set__0_5_2",
        url = "https://crates.io/api/v1/crates/bit-set/0.5.2/download",
        type = "tar.gz",
        sha256 = "6e11e16035ea35e4e5997b393eacbf6f63983188f7a2ad25bfb13465f5ad59de",
        strip_prefix = "bit-set-0.5.2",
        build_file = Label("//cargo/remote:BUILD.bit-set-0.5.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bit_vec__0_6_3",
        url = "https://crates.io/api/v1/crates/bit-vec/0.6.3/download",
        type = "tar.gz",
        sha256 = "349f9b6a179ed607305526ca489b34ad0a41aed5f7980fa90eb03160b69598fb",
        strip_prefix = "bit-vec-0.6.3",
        build_file = Label("//cargo/remote:BUILD.bit-vec-0.6.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bitflags__1_3_2",
        url = "https://crates.io/api/v1/crates/bitflags/1.3.2/download",
        type = "tar.gz",
        sha256 = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a",
        strip_prefix = "bitflags-1.3.2",
        build_file = Label("//cargo/remote:BUILD.bitflags-1.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bstr__0_2_17",
        url = "https://crates.io/api/v1/crates/bstr/0.2.17/download",
        type = "tar.gz",
        sha256 = "ba3569f383e8f1598449f1a423e72e99569137b47740b1da11ef19af3d5c3223",
        strip_prefix = "bstr-0.2.17",
        build_file = Label("//cargo/remote:BUILD.bstr-0.2.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bytecount__0_6_2",
        url = "https://crates.io/api/v1/crates/bytecount/0.6.2/download",
        type = "tar.gz",
        sha256 = "72feb31ffc86498dacdbd0fcebb56138e7177a8cc5cea4516031d15ae85a742e",
        strip_prefix = "bytecount-0.6.2",
        build_file = Label("//cargo/remote:BUILD.bytecount-0.6.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__byteorder__1_4_3",
        url = "https://crates.io/api/v1/crates/byteorder/1.4.3/download",
        type = "tar.gz",
        sha256 = "14c189c53d098945499cdfa7ecc63567cf3886b3332b312a5b4585d8d3a6a610",
        strip_prefix = "byteorder-1.4.3",
        build_file = Label("//cargo/remote:BUILD.byteorder-1.4.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cfg_if__1_0_0",
        url = "https://crates.io/api/v1/crates/cfg-if/1.0.0/download",
        type = "tar.gz",
        sha256 = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd",
        strip_prefix = "cfg-if-1.0.0",
        build_file = Label("//cargo/remote:BUILD.cfg-if-1.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clap__2_34_0",
        url = "https://crates.io/api/v1/crates/clap/2.34.0/download",
        type = "tar.gz",
        sha256 = "a0610544180c38b88101fecf2dd634b174a62eef6946f84dfc6a7127512b381c",
        strip_prefix = "clap-2.34.0",
        build_file = Label("//cargo/remote:BUILD.clap-2.34.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__colored__2_0_0",
        url = "https://crates.io/api/v1/crates/colored/2.0.0/download",
        type = "tar.gz",
        sha256 = "b3616f750b84d8f0de8a58bda93e08e2a81ad3f523089b05f1dffecab48c6cbd",
        strip_prefix = "colored-2.0.0",
        build_file = Label("//cargo/remote:BUILD.colored-2.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__config__0_10_1",
        url = "https://crates.io/api/v1/crates/config/0.10.1/download",
        type = "tar.gz",
        sha256 = "19b076e143e1d9538dde65da30f8481c2a6c44040edb8e02b9bf1351edb92ce3",
        strip_prefix = "config-0.10.1",
        build_file = Label("//cargo/remote:BUILD.config-0.10.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam_utils__0_8_6",
        url = "https://crates.io/api/v1/crates/crossbeam-utils/0.8.6/download",
        type = "tar.gz",
        sha256 = "cfcae03edb34f947e64acdb1c33ec169824e20657e9ecb61cef6c8c74dcb8120",
        strip_prefix = "crossbeam-utils-0.8.6",
        build_file = Label("//cargo/remote:BUILD.crossbeam-utils-0.8.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__difflib__0_4_0",
        url = "https://crates.io/api/v1/crates/difflib/0.4.0/download",
        type = "tar.gz",
        sha256 = "6184e33543162437515c2e2b48714794e37845ec9851711914eec9d308f6ebe8",
        strip_prefix = "difflib-0.4.0",
        build_file = Label("//cargo/remote:BUILD.difflib-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__dirs_next__2_0_0",
        url = "https://crates.io/api/v1/crates/dirs-next/2.0.0/download",
        type = "tar.gz",
        sha256 = "b98cf8ebf19c3d1b223e151f99a4f9f0690dca41414773390fc824184ac833e1",
        strip_prefix = "dirs-next-2.0.0",
        build_file = Label("//cargo/remote:BUILD.dirs-next-2.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__dirs_sys_next__0_1_2",
        url = "https://crates.io/api/v1/crates/dirs-sys-next/0.1.2/download",
        type = "tar.gz",
        sha256 = "4ebda144c4fe02d1f7ea1a7d9641b6fc6b580adcfa024ae48797ecdeb6825b4d",
        strip_prefix = "dirs-sys-next-0.1.2",
        build_file = Label("//cargo/remote:BUILD.dirs-sys-next-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__doc_comment__0_3_3",
        url = "https://crates.io/api/v1/crates/doc-comment/0.3.3/download",
        type = "tar.gz",
        sha256 = "fea41bba32d969b513997752735605054bc0dfa92b4c56bf1189f2e174be7a10",
        strip_prefix = "doc-comment-0.3.3",
        build_file = Label("//cargo/remote:BUILD.doc-comment-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__either__1_6_1",
        url = "https://crates.io/api/v1/crates/either/1.6.1/download",
        type = "tar.gz",
        sha256 = "e78d4f1cc4ae33bbfc157ed5d5a5ef3bc29227303d595861deb238fcec4e9457",
        strip_prefix = "either-1.6.1",
        build_file = Label("//cargo/remote:BUILD.either-1.6.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fastrand__1_6_0",
        url = "https://crates.io/api/v1/crates/fastrand/1.6.0/download",
        type = "tar.gz",
        sha256 = "779d043b6a0b90cc4c0ed7ee380a6504394cee7efd7db050e3774eee387324b2",
        strip_prefix = "fastrand-1.6.0",
        build_file = Label("//cargo/remote:BUILD.fastrand-1.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fixedbitset__0_2_0",
        url = "https://crates.io/api/v1/crates/fixedbitset/0.2.0/download",
        type = "tar.gz",
        sha256 = "37ab347416e802de484e4d03c7316c48f1ecb56574dfd4a46a80f173ce1de04d",
        strip_prefix = "fixedbitset-0.2.0",
        build_file = Label("//cargo/remote:BUILD.fixedbitset-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__float_ord__0_2_0",
        url = "https://crates.io/api/v1/crates/float-ord/0.2.0/download",
        type = "tar.gz",
        sha256 = "7bad48618fdb549078c333a7a8528acb57af271d0433bdecd523eb620628364e",
        strip_prefix = "float-ord-0.2.0",
        build_file = Label("//cargo/remote:BUILD.float-ord-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fnv__1_0_7",
        url = "https://crates.io/api/v1/crates/fnv/1.0.7/download",
        type = "tar.gz",
        sha256 = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1",
        strip_prefix = "fnv-1.0.7",
        build_file = Label("//cargo/remote:BUILD.fnv-1.0.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__getrandom__0_2_4",
        url = "https://crates.io/api/v1/crates/getrandom/0.2.4/download",
        type = "tar.gz",
        sha256 = "418d37c8b1d42553c93648be529cb70f920d3baf8ef469b74b9638df426e0b4c",
        strip_prefix = "getrandom-0.2.4",
        build_file = Label("//cargo/remote:BUILD.getrandom-0.2.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__globset__0_4_8",
        url = "https://crates.io/api/v1/crates/globset/0.4.8/download",
        type = "tar.gz",
        sha256 = "10463d9ff00a2a068db14231982f5132edebad0d7660cd956a1c30292dbcbfbd",
        strip_prefix = "globset-0.4.8",
        build_file = Label("//cargo/remote:BUILD.globset-0.4.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hashbrown__0_11_2",
        url = "https://crates.io/api/v1/crates/hashbrown/0.11.2/download",
        type = "tar.gz",
        sha256 = "ab5ef0d4909ef3724cc8cce6ccc8572c5c817592e9285f5464f8e86f8bd3726e",
        strip_prefix = "hashbrown-0.11.2",
        build_file = Label("//cargo/remote:BUILD.hashbrown-0.11.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hermit_abi__0_1_19",
        url = "https://crates.io/api/v1/crates/hermit-abi/0.1.19/download",
        type = "tar.gz",
        sha256 = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33",
        strip_prefix = "hermit-abi-0.1.19",
        build_file = Label("//cargo/remote:BUILD.hermit-abi-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ignore__0_4_18",
        url = "https://crates.io/api/v1/crates/ignore/0.4.18/download",
        type = "tar.gz",
        sha256 = "713f1b139373f96a2e0ce3ac931cd01ee973c3c5dd7c40c0c2efe96ad2b6751d",
        strip_prefix = "ignore-0.4.18",
        build_file = Label("//cargo/remote:BUILD.ignore-0.4.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indexmap__1_8_0",
        url = "https://crates.io/api/v1/crates/indexmap/1.8.0/download",
        type = "tar.gz",
        sha256 = "282a6247722caba404c065016bbfa522806e51714c34f5dfc3e4a3a46fcb4223",
        strip_prefix = "indexmap-1.8.0",
        build_file = Label("//cargo/remote:BUILD.indexmap-1.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__instant__0_1_12",
        url = "https://crates.io/api/v1/crates/instant/0.1.12/download",
        type = "tar.gz",
        sha256 = "7a5bbe824c507c5da5956355e86a746d82e0e1464f65d862cc5e71da70e94b2c",
        strip_prefix = "instant-0.1.12",
        build_file = Label("//cargo/remote:BUILD.instant-0.1.12.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itertools__0_10_3",
        url = "https://crates.io/api/v1/crates/itertools/0.10.3/download",
        type = "tar.gz",
        sha256 = "a9a9d19fa1e79b6215ff29b9d6880b706147f16e9b1dbb1e4e5947b5b02bc5e3",
        strip_prefix = "itertools-0.10.3",
        build_file = Label("//cargo/remote:BUILD.itertools-0.10.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itoa__1_0_1",
        url = "https://crates.io/api/v1/crates/itoa/1.0.1/download",
        type = "tar.gz",
        sha256 = "1aab8fc367588b89dcee83ab0fd66b72b50b72fa1904d7095045ace2b0c81c35",
        strip_prefix = "itoa-1.0.1",
        build_file = Label("//cargo/remote:BUILD.itoa-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lazy_static__1_4_0",
        url = "https://crates.io/api/v1/crates/lazy_static/1.4.0/download",
        type = "tar.gz",
        sha256 = "e2abad23fbc42b3700f2f279844dc832adb2b2eb069b2df918f455c4e18cc646",
        strip_prefix = "lazy_static-1.4.0",
        build_file = Label("//cargo/remote:BUILD.lazy_static-1.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lexical_core__0_7_6",
        url = "https://crates.io/api/v1/crates/lexical-core/0.7.6/download",
        type = "tar.gz",
        sha256 = "6607c62aa161d23d17a9072cc5da0be67cdfc89d3afb1e8d9c842bebc2525ffe",
        strip_prefix = "lexical-core-0.7.6",
        build_file = Label("//cargo/remote:BUILD.lexical-core-0.7.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libc__0_2_113",
        url = "https://crates.io/api/v1/crates/libc/0.2.113/download",
        type = "tar.gz",
        sha256 = "eef78b64d87775463c549fbd80e19249ef436ea3bf1de2a1eb7e717ec7fab1e9",
        strip_prefix = "libc-0.2.113",
        build_file = Label("//cargo/remote:BUILD.libc-0.2.113.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__linked_hash_map__0_3_0",
        url = "https://crates.io/api/v1/crates/linked-hash-map/0.3.0/download",
        type = "tar.gz",
        sha256 = "6d262045c5b87c0861b3f004610afd0e2c851e2908d08b6c870cbb9d5f494ecd",
        strip_prefix = "linked-hash-map-0.3.0",
        build_file = Label("//cargo/remote:BUILD.linked-hash-map-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__linked_hash_map__0_5_4",
        url = "https://crates.io/api/v1/crates/linked-hash-map/0.5.4/download",
        type = "tar.gz",
        sha256 = "7fb9b38af92608140b86b693604b9ffcc5824240a484d1ecd4795bacb2fe88f3",
        strip_prefix = "linked-hash-map-0.5.4",
        build_file = Label("//cargo/remote:BUILD.linked-hash-map-0.5.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log__0_4_14",
        url = "https://crates.io/api/v1/crates/log/0.4.14/download",
        type = "tar.gz",
        sha256 = "51b9bbe6c47d51fc3e1a9b945965946b4c44142ab8792c50835a980d362c2710",
        strip_prefix = "log-0.4.14",
        build_file = Label("//cargo/remote:BUILD.log-0.4.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memchr__2_4_1",
        url = "https://crates.io/api/v1/crates/memchr/2.4.1/download",
        type = "tar.gz",
        sha256 = "308cc39be01b73d0d18f82a0e7b2a3df85245f84af96fdddc5d202d27e47b86a",
        strip_prefix = "memchr-2.4.1",
        build_file = Label("//cargo/remote:BUILD.memchr-2.4.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__nom__5_1_2",
        url = "https://crates.io/api/v1/crates/nom/5.1.2/download",
        type = "tar.gz",
        sha256 = "ffb4262d26ed83a1c0a33a38fe2bb15797329c85770da05e6b828ddb782627af",
        strip_prefix = "nom-5.1.2",
        build_file = Label("//cargo/remote:BUILD.nom-5.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__nom_locate__2_1_0",
        url = "https://crates.io/api/v1/crates/nom_locate/2.1.0/download",
        type = "tar.gz",
        sha256 = "a67484adf5711f94f2f28b653bf231bff8e438be33bf5b0f35935a0db4f618a2",
        strip_prefix = "nom_locate-2.1.0",
        build_file = Label("//cargo/remote:BUILD.nom_locate-2.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_traits__0_1_43",
        url = "https://crates.io/api/v1/crates/num-traits/0.1.43/download",
        type = "tar.gz",
        sha256 = "92e5113e9fd4cc14ded8e499429f396a20f98c772a47cc8622a736e1ec843c31",
        strip_prefix = "num-traits-0.1.43",
        build_file = Label("//cargo/remote:BUILD.num-traits-0.1.43.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_traits__0_2_14",
        url = "https://crates.io/api/v1/crates/num-traits/0.2.14/download",
        type = "tar.gz",
        sha256 = "9a64b1ec5cda2586e284722486d802acf1f7dbdc623e2bfc57e65ca1cd099290",
        strip_prefix = "num-traits-0.2.14",
        build_file = Label("//cargo/remote:BUILD.num-traits-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__numtoa__0_1_0",
        url = "https://crates.io/api/v1/crates/numtoa/0.1.0/download",
        type = "tar.gz",
        sha256 = "b8f8bdf33df195859076e54ab11ee78a1b208382d3a26ec40d142ffc1ecc49ef",
        strip_prefix = "numtoa-0.1.0",
        build_file = Label("//cargo/remote:BUILD.numtoa-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__once_cell__1_9_0",
        url = "https://crates.io/api/v1/crates/once_cell/1.9.0/download",
        type = "tar.gz",
        sha256 = "da32515d9f6e6e489d7bc9d84c71b060db7247dc035bbe44eac88cf87486d8d5",
        strip_prefix = "once_cell-1.9.0",
        build_file = Label("//cargo/remote:BUILD.once_cell-1.9.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__petgraph__0_5_1",
        url = "https://crates.io/api/v1/crates/petgraph/0.5.1/download",
        type = "tar.gz",
        sha256 = "467d164a6de56270bd7c4d070df81d07beace25012d5103ced4e9ff08d6afdb7",
        strip_prefix = "petgraph-0.5.1",
        build_file = Label("//cargo/remote:BUILD.petgraph-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ppv_lite86__0_2_16",
        url = "https://crates.io/api/v1/crates/ppv-lite86/0.2.16/download",
        type = "tar.gz",
        sha256 = "eb9f9e6e233e5c4a35559a617bf40a4ec447db2e84c20b55a6f83167b7e57872",
        strip_prefix = "ppv-lite86-0.2.16",
        build_file = Label("//cargo/remote:BUILD.ppv-lite86-0.2.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__predicates__2_1_1",
        url = "https://crates.io/api/v1/crates/predicates/2.1.1/download",
        type = "tar.gz",
        sha256 = "a5aab5be6e4732b473071984b3164dbbfb7a3674d30ea5ff44410b6bcd960c3c",
        strip_prefix = "predicates-2.1.1",
        build_file = Label("//cargo/remote:BUILD.predicates-2.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__predicates_core__1_0_3",
        url = "https://crates.io/api/v1/crates/predicates-core/1.0.3/download",
        type = "tar.gz",
        sha256 = "da1c2388b1513e1b605fcec39a95e0a9e8ef088f71443ef37099fa9ae6673fcb",
        strip_prefix = "predicates-core-1.0.3",
        build_file = Label("//cargo/remote:BUILD.predicates-core-1.0.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__predicates_tree__1_0_5",
        url = "https://crates.io/api/v1/crates/predicates-tree/1.0.5/download",
        type = "tar.gz",
        sha256 = "4d86de6de25020a36c6d3643a86d9a6a9f552107c0559c60ea03551b5e16c032",
        strip_prefix = "predicates-tree-1.0.5",
        build_file = Label("//cargo/remote:BUILD.predicates-tree-1.0.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__1_0_36",
        url = "https://crates.io/api/v1/crates/proc-macro2/1.0.36/download",
        type = "tar.gz",
        sha256 = "c7342d5883fbccae1cc37a2353b09c87c9b0f3afd73f5fb9bba687a1f733b029",
        strip_prefix = "proc-macro2-1.0.36",
        build_file = Label("//cargo/remote:BUILD.proc-macro2-1.0.36.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proptest__1_0_0",
        url = "https://crates.io/api/v1/crates/proptest/1.0.0/download",
        type = "tar.gz",
        sha256 = "1e0d9cc07f18492d879586c92b485def06bc850da3118075cd45d50e9c95b0e5",
        strip_prefix = "proptest-1.0.0",
        build_file = Label("//cargo/remote:BUILD.proptest-1.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quick_error__1_2_3",
        url = "https://crates.io/api/v1/crates/quick-error/1.2.3/download",
        type = "tar.gz",
        sha256 = "a1d01941d82fa2ab50be1e79e6714289dd7cde78eba4c074bc5a4374f650dfe0",
        strip_prefix = "quick-error-1.2.3",
        build_file = Label("//cargo/remote:BUILD.quick-error-1.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quick_error__2_0_1",
        url = "https://crates.io/api/v1/crates/quick-error/2.0.1/download",
        type = "tar.gz",
        sha256 = "a993555f31e5a609f617c12db6250dedcac1b0a85076912c436e6fc9b2c8e6a3",
        strip_prefix = "quick-error-2.0.1",
        build_file = Label("//cargo/remote:BUILD.quick-error-2.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__1_0_14",
        url = "https://crates.io/api/v1/crates/quote/1.0.14/download",
        type = "tar.gz",
        sha256 = "47aa80447ce4daf1717500037052af176af5d38cc3e571d9ec1c7353fc10c87d",
        strip_prefix = "quote-1.0.14",
        build_file = Label("//cargo/remote:BUILD.quote-1.0.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_8_4",
        url = "https://crates.io/api/v1/crates/rand/0.8.4/download",
        type = "tar.gz",
        sha256 = "2e7573632e6454cf6b99d7aac4ccca54be06da05aca2ef7423d22d27d4d4bcd8",
        strip_prefix = "rand-0.8.4",
        build_file = Label("//cargo/remote:BUILD.rand-0.8.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_chacha__0_3_1",
        url = "https://crates.io/api/v1/crates/rand_chacha/0.3.1/download",
        type = "tar.gz",
        sha256 = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88",
        strip_prefix = "rand_chacha-0.3.1",
        build_file = Label("//cargo/remote:BUILD.rand_chacha-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_core__0_6_3",
        url = "https://crates.io/api/v1/crates/rand_core/0.6.3/download",
        type = "tar.gz",
        sha256 = "d34f1408f55294453790c48b2f1ebbb1c5b4b7563eb1f418bcfcfdbb06ebb4e7",
        strip_prefix = "rand_core-0.6.3",
        build_file = Label("//cargo/remote:BUILD.rand_core-0.6.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_xorshift__0_3_0",
        url = "https://crates.io/api/v1/crates/rand_xorshift/0.3.0/download",
        type = "tar.gz",
        sha256 = "d25bf25ec5ae4a3f1b92f929810509a2f53d7dca2f50b794ff57e3face536c8f",
        strip_prefix = "rand_xorshift-0.3.0",
        build_file = Label("//cargo/remote:BUILD.rand_xorshift-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_syscall__0_2_10",
        url = "https://crates.io/api/v1/crates/redox_syscall/0.2.10/download",
        type = "tar.gz",
        sha256 = "8383f39639269cde97d255a32bdb68c047337295414940c68bdd30c2e13203ff",
        strip_prefix = "redox_syscall-0.2.10",
        build_file = Label("//cargo/remote:BUILD.redox_syscall-0.2.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_termios__0_1_2",
        url = "https://crates.io/api/v1/crates/redox_termios/0.1.2/download",
        type = "tar.gz",
        sha256 = "8440d8acb4fd3d277125b4bd01a6f38aee8d814b3b5fc09b3f2b825d37d3fe8f",
        strip_prefix = "redox_termios-0.1.2",
        build_file = Label("//cargo/remote:BUILD.redox_termios-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_users__0_4_0",
        url = "https://crates.io/api/v1/crates/redox_users/0.4.0/download",
        type = "tar.gz",
        sha256 = "528532f3d801c87aec9def2add9ca802fe569e44a544afe633765267840abe64",
        strip_prefix = "redox_users-0.4.0",
        build_file = Label("//cargo/remote:BUILD.redox_users-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex__1_5_4",
        url = "https://crates.io/api/v1/crates/regex/1.5.4/download",
        type = "tar.gz",
        sha256 = "d07a8629359eb56f1e2fb1652bb04212c072a87ba68546a04065d525673ac461",
        strip_prefix = "regex-1.5.4",
        build_file = Label("//cargo/remote:BUILD.regex-1.5.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex_automata__0_1_10",
        url = "https://crates.io/api/v1/crates/regex-automata/0.1.10/download",
        type = "tar.gz",
        sha256 = "6c230d73fb8d8c1b9c0b3135c5142a8acee3a0558fb8db5cf1cb65f8d7862132",
        strip_prefix = "regex-automata-0.1.10",
        build_file = Label("//cargo/remote:BUILD.regex-automata-0.1.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex_syntax__0_6_25",
        url = "https://crates.io/api/v1/crates/regex-syntax/0.6.25/download",
        type = "tar.gz",
        sha256 = "f497285884f3fcff424ffc933e56d7cbca511def0c9831a7f9b5f6153e3cc89b",
        strip_prefix = "regex-syntax-0.6.25",
        build_file = Label("//cargo/remote:BUILD.regex-syntax-0.6.25.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__remove_dir_all__0_5_3",
        url = "https://crates.io/api/v1/crates/remove_dir_all/0.5.3/download",
        type = "tar.gz",
        sha256 = "3acd125665422973a33ac9d3dd2df85edad0f4ae9b00dafb1a05e43a9f5ef8e7",
        strip_prefix = "remove_dir_all-0.5.3",
        build_file = Label("//cargo/remote:BUILD.remove_dir_all-0.5.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rental__0_5_6",
        url = "https://crates.io/api/v1/crates/rental/0.5.6/download",
        type = "tar.gz",
        sha256 = "cc89fe2acac36d212474d138aaf939c04a82df5b61d07011571ebce5aef81f2e",
        strip_prefix = "rental-0.5.6",
        build_file = Label("//cargo/remote:BUILD.rental-0.5.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rental_impl__0_5_5",
        url = "https://crates.io/api/v1/crates/rental-impl/0.5.5/download",
        type = "tar.gz",
        sha256 = "475e68978dc5b743f2f40d8e0a8fdc83f1c5e78cbf4b8fa5e74e73beebc340de",
        strip_prefix = "rental-impl-0.5.5",
        build_file = Label("//cargo/remote:BUILD.rental-impl-0.5.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rust_ini__0_13_0",
        url = "https://crates.io/api/v1/crates/rust-ini/0.13.0/download",
        type = "tar.gz",
        sha256 = "3e52c148ef37f8c375d49d5a73aa70713125b7f19095948a923f80afdeb22ec2",
        strip_prefix = "rust-ini-0.13.0",
        build_file = Label("//cargo/remote:BUILD.rust-ini-0.13.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rusty_fork__0_3_0",
        url = "https://crates.io/api/v1/crates/rusty-fork/0.3.0/download",
        type = "tar.gz",
        sha256 = "cb3dcc6e454c328bb824492db107ab7c0ae8fcffe4ad210136ef014458c1bc4f",
        strip_prefix = "rusty-fork-0.3.0",
        build_file = Label("//cargo/remote:BUILD.rusty-fork-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ryu__1_0_9",
        url = "https://crates.io/api/v1/crates/ryu/1.0.9/download",
        type = "tar.gz",
        sha256 = "73b4b750c782965c211b42f022f59af1fbceabdd026623714f104152f1ec149f",
        strip_prefix = "ryu-1.0.9",
        build_file = Label("//cargo/remote:BUILD.ryu-1.0.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__same_file__1_0_6",
        url = "https://crates.io/api/v1/crates/same-file/1.0.6/download",
        type = "tar.gz",
        sha256 = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502",
        strip_prefix = "same-file-1.0.6",
        build_file = Label("//cargo/remote:BUILD.same-file-1.0.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde__0_8_23",
        url = "https://crates.io/api/v1/crates/serde/0.8.23/download",
        type = "tar.gz",
        sha256 = "9dad3f759919b92c3068c696c15c3d17238234498bbdcc80f2c469606f948ac8",
        strip_prefix = "serde-0.8.23",
        build_file = Label("//cargo/remote:BUILD.serde-0.8.23.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde__1_0_134",
        url = "https://crates.io/api/v1/crates/serde/1.0.134/download",
        type = "tar.gz",
        sha256 = "96b3c34c1690edf8174f5b289a336ab03f568a4460d8c6df75f2f3a692b3bc6a",
        strip_prefix = "serde-1.0.134",
        build_file = Label("//cargo/remote:BUILD.serde-1.0.134.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_hjson__0_9_1",
        url = "https://crates.io/api/v1/crates/serde-hjson/0.9.1/download",
        type = "tar.gz",
        sha256 = "6a3a4e0ea8a88553209f6cc6cfe8724ecad22e1acf372793c27d995290fe74f8",
        strip_prefix = "serde-hjson-0.9.1",
        build_file = Label("//cargo/remote:BUILD.serde-hjson-0.9.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_derive__1_0_134",
        url = "https://crates.io/api/v1/crates/serde_derive/1.0.134/download",
        type = "tar.gz",
        sha256 = "784ed1fbfa13fe191077537b0d70ec8ad1e903cfe04831da608aa36457cb653d",
        strip_prefix = "serde_derive-1.0.134",
        build_file = Label("//cargo/remote:BUILD.serde_derive-1.0.134.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_json__1_0_75",
        url = "https://crates.io/api/v1/crates/serde_json/1.0.75/download",
        type = "tar.gz",
        sha256 = "c059c05b48c5c0067d4b4b2b4f0732dd65feb52daf7e0ea09cd87e7dadc1af79",
        strip_prefix = "serde_json-1.0.75",
        build_file = Label("//cargo/remote:BUILD.serde_json-1.0.75.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_test__0_8_23",
        url = "https://crates.io/api/v1/crates/serde_test/0.8.23/download",
        type = "tar.gz",
        sha256 = "110b3dbdf8607ec493c22d5d947753282f3bae73c0f56d322af1e8c78e4c23d5",
        strip_prefix = "serde_test-0.8.23",
        build_file = Label("//cargo/remote:BUILD.serde_test-0.8.23.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__shellexpand__2_1_0",
        url = "https://crates.io/api/v1/crates/shellexpand/2.1.0/download",
        type = "tar.gz",
        sha256 = "83bdb7831b2d85ddf4a7b148aa19d0587eddbe8671a436b7bd1182eaad0f2829",
        strip_prefix = "shellexpand-2.1.0",
        build_file = Label("//cargo/remote:BUILD.shellexpand-2.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__source_span__2_7_0",
        url = "https://crates.io/api/v1/crates/source-span/2.7.0/download",
        type = "tar.gz",
        sha256 = "a3afeb8b7a1ae2e8721f2193cc2291c9e00dd68511907fd8b807e2c8d42caa3c",
        strip_prefix = "source-span-2.7.0",
        build_file = Label("//cargo/remote:BUILD.source-span-2.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__stable_deref_trait__1_2_0",
        url = "https://crates.io/api/v1/crates/stable_deref_trait/1.2.0/download",
        type = "tar.gz",
        sha256 = "a8f112729512f8e442d81f95a8a7ddf2b7c6b8a1a6f509a95864142b30cab2d3",
        strip_prefix = "stable_deref_trait-1.2.0",
        build_file = Label("//cargo/remote:BUILD.stable_deref_trait-1.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__static_assertions__1_1_0",
        url = "https://crates.io/api/v1/crates/static_assertions/1.1.0/download",
        type = "tar.gz",
        sha256 = "a2eb9349b6444b326872e140eb1cf5e7c522154d69e7a0ffb0fb81c06b37543f",
        strip_prefix = "static_assertions-1.1.0",
        build_file = Label("//cargo/remote:BUILD.static_assertions-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__strsim__0_10_0",
        url = "https://crates.io/api/v1/crates/strsim/0.10.0/download",
        type = "tar.gz",
        sha256 = "73473c0e59e6d5812c5dfe2a064a6444949f089e20eec9a2e5506596494e4623",
        strip_prefix = "strsim-0.10.0",
        build_file = Label("//cargo/remote:BUILD.strsim-0.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__syn__1_0_86",
        url = "https://crates.io/api/v1/crates/syn/1.0.86/download",
        type = "tar.gz",
        sha256 = "8a65b3f4ffa0092e9887669db0eae07941f023991ab58ea44da8fe8e2d511c6b",
        strip_prefix = "syn-1.0.86",
        build_file = Label("//cargo/remote:BUILD.syn-1.0.86.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tempfile__3_3_0",
        url = "https://crates.io/api/v1/crates/tempfile/3.3.0/download",
        type = "tar.gz",
        sha256 = "5cdb1ef4eaeeaddc8fbd371e5017057064af0911902ef36b39801f67cc6d79e4",
        strip_prefix = "tempfile-3.3.0",
        build_file = Label("//cargo/remote:BUILD.tempfile-3.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termion__1_5_6",
        url = "https://crates.io/api/v1/crates/termion/1.5.6/download",
        type = "tar.gz",
        sha256 = "077185e2eac69c3f8379a4298e1e07cd36beb962290d4a51199acf0fdc10607e",
        strip_prefix = "termion-1.5.6",
        build_file = Label("//cargo/remote:BUILD.termion-1.5.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termtree__0_2_4",
        url = "https://crates.io/api/v1/crates/termtree/0.2.4/download",
        type = "tar.gz",
        sha256 = "507e9898683b6c43a9aa55b64259b721b52ba226e0f3779137e50ad114a4c90b",
        strip_prefix = "termtree-0.2.4",
        build_file = Label("//cargo/remote:BUILD.termtree-0.2.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__textwrap__0_11_0",
        url = "https://crates.io/api/v1/crates/textwrap/0.11.0/download",
        type = "tar.gz",
        sha256 = "d326610f408c7a4eb6f51c37c330e496b08506c9457c9d34287ecc38809fb060",
        strip_prefix = "textwrap-0.11.0",
        build_file = Label("//cargo/remote:BUILD.textwrap-0.11.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thread_local__1_1_3",
        url = "https://crates.io/api/v1/crates/thread_local/1.1.3/download",
        type = "tar.gz",
        sha256 = "8018d24e04c95ac8790716a5987d0fec4f8b27249ffa0f7d33f1369bdfb88cbd",
        strip_prefix = "thread_local-1.1.3",
        build_file = Label("//cargo/remote:BUILD.thread_local-1.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__toml__0_5_8",
        url = "https://crates.io/api/v1/crates/toml/0.5.8/download",
        type = "tar.gz",
        sha256 = "a31142970826733df8241ef35dc040ef98c679ab14d7c3e54d827099b3acecaa",
        strip_prefix = "toml-0.5.8",
        build_file = Label("//cargo/remote:BUILD.toml-0.5.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_width__0_1_9",
        url = "https://crates.io/api/v1/crates/unicode-width/0.1.9/download",
        type = "tar.gz",
        sha256 = "3ed742d4ea2bd1176e236172c8429aaf54486e7ac098db29ffe6529e0ce50973",
        strip_prefix = "unicode-width-0.1.9",
        build_file = Label("//cargo/remote:BUILD.unicode-width-0.1.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_xid__0_2_2",
        url = "https://crates.io/api/v1/crates/unicode-xid/0.2.2/download",
        type = "tar.gz",
        sha256 = "8ccb82d61f80a663efe1f787a51b16b5a51e3314d6ac365b08639f52387b33f3",
        strip_prefix = "unicode-xid-0.2.2",
        build_file = Label("//cargo/remote:BUILD.unicode-xid-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__version_check__0_9_4",
        url = "https://crates.io/api/v1/crates/version_check/0.9.4/download",
        type = "tar.gz",
        sha256 = "49874b5167b65d7193b8aba1567f5c7d93d001cafc34600cee003eda787e483f",
        strip_prefix = "version_check-0.9.4",
        build_file = Label("//cargo/remote:BUILD.version_check-0.9.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wait_timeout__0_2_0",
        url = "https://crates.io/api/v1/crates/wait-timeout/0.2.0/download",
        type = "tar.gz",
        sha256 = "9f200f5b12eb75f8c1ed65abd4b2db8a6e1b138a20de009dacee265a2498f3f6",
        strip_prefix = "wait-timeout-0.2.0",
        build_file = Label("//cargo/remote:BUILD.wait-timeout-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__walkdir__2_3_2",
        url = "https://crates.io/api/v1/crates/walkdir/2.3.2/download",
        type = "tar.gz",
        sha256 = "808cf2735cd4b6866113f648b791c6adc5714537bc222d9347bb203386ffda56",
        strip_prefix = "walkdir-2.3.2",
        build_file = Label("//cargo/remote:BUILD.walkdir-2.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasi__0_10_2_wasi_snapshot_preview1",
        url = "https://crates.io/api/v1/crates/wasi/0.10.2+wasi-snapshot-preview1/download",
        type = "tar.gz",
        sha256 = "fd6fbd9a79829dd1ad0cc20627bf1ed606756a7f77edff7b66b7064f9cb327c6",
        strip_prefix = "wasi-0.10.2+wasi-snapshot-preview1",
        build_file = Label("//cargo/remote:BUILD.wasi-0.10.2+wasi-snapshot-preview1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi__0_3_9",
        url = "https://crates.io/api/v1/crates/winapi/0.3.9/download",
        type = "tar.gz",
        sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419",
        strip_prefix = "winapi-0.3.9",
        build_file = Label("//cargo/remote:BUILD.winapi-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-i686-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//cargo/remote:BUILD.winapi-i686-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_util__0_1_5",
        url = "https://crates.io/api/v1/crates/winapi-util/0.1.5/download",
        type = "tar.gz",
        sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178",
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("//cargo/remote:BUILD.winapi-util-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-x86_64-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//cargo/remote:BUILD.winapi-x86_64-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__yaml_rust__0_4_5",
        url = "https://crates.io/api/v1/crates/yaml-rust/0.4.5/download",
        type = "tar.gz",
        sha256 = "56c1936c4cc7a1c9ab21a1ebb602eb942ba868cbd44a99cb7cdc5892335e1c85",
        strip_prefix = "yaml-rust-0.4.5",
        build_file = Label("//cargo/remote:BUILD.yaml-rust-0.4.5.bazel"),
    )
