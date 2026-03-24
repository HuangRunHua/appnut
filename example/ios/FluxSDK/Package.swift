// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FluxSDK",
    platforms: [.iOS(.v16)],
    products: [
        .library(name: "FluxSDK", targets: ["FluxSDK"]),
    ],
    targets: [
        .binaryTarget(
            name: "FluxFFI",
            path: "../../../target/ios/FluxFFI.xcframework"
        ),
        .target(
            name: "FluxSDK",
            dependencies: ["FluxFFI"],
            path: "Sources"
        ),
    ]
)
