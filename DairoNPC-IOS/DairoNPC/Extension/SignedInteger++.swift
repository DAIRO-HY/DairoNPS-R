
import Foundation
extension SignedInteger{
    
    /**
     * 将整形数据转成文件大小格式的字符串
     */
    var readableSize: String {
        let kb = 1024.0
        let mb = kb * 1024
        let gb = mb * 1024
        let tb = gb * 1024

        let value = Double(self)

        switch value {
        case tb...:
            return String(format: "%.2fTB", value / tb)
        case gb...:
            return String(format: "%.2fGB", value / gb)
        case mb...:
            return String(format: "%.2fMB", value / mb)
        case kb...:
            return String(format: "%.2fKB", value / kb)
        default:
            return "\(self)B"
        }
    }
}
