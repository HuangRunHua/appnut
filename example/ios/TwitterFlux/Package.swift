// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "TwitterFlux",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "TwitterFlux", targets: ["TwitterFlux"]),
    ],
    dependencies: [
        .package(path: "../FluxSDK"),
    ],
    targets: [
        .executableTarget(
            name: "TwitterFlux",
            dependencies: [
                .product(name: "FluxSDK", package: "FluxSDK"),
            ],
            path: "TwitterFlux"
        ),
    ]
)
