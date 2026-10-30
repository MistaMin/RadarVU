// Lists all windows, including those of a hidden app, of one process as JSON: id, title, bounds.
//   swiftc -O winlist.swift -o winlist && ./winlist <pid>
// Titles are only visible with Screen Recording permission for the calling app (macOS 10.15+);
// without it "title" is empty and screencapture -l captures only the desktop behind the window.
import CoreGraphics
import Foundation

let pid = CommandLine.arguments.count > 1 ? Int(CommandLine.arguments[1]) ?? -1 : -1
let infos = CGWindowListCopyWindowInfo([.optionAll, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
var out: [[String: Any]] = []
for w in infos {
	guard let owner = w[kCGWindowOwnerPID as String] as? Int, pid < 0 || owner == pid else { continue }
	let b = w[kCGWindowBounds as String] as? [String: Double] ?? [:]
	out.append([
		"id": w[kCGWindowNumber as String] as? Int ?? 0,
		"pid": owner,
		"owner": w[kCGWindowOwnerName as String] as? String ?? "",
		"title": w[kCGWindowName as String] as? String ?? "",
		"layer": w[kCGWindowLayer as String] as? Int ?? 0,
		"x": b["X"] ?? 0, "y": b["Y"] ?? 0, "width": b["Width"] ?? 0, "height": b["Height"] ?? 0,
	])
}
let data = try JSONSerialization.data(withJSONObject: out, options: [.prettyPrinted])
FileHandle.standardOutput.write(data)
