import Foundation
class NpcRepository {
    
    /**
     * 配置存储key
     */
    private static let settingKey = "setting"
    
    /**
     * 标记是否是打开状态的
     */
    private static let isOpenedKey = "isOpened"
    
    /**
     * 是否已经配置标记存储key
     */
    private static let isSetKey = "isSet"
    
    /**
     * 从缓存加载
     */
    static func loadSetting() -> NpcSetting {
        if let jsonData = UserDefaults.standard.data(forKey: settingKey){
            if let npcSetting = try? JSONDecoder().decode(NpcSetting.self, from: jsonData){
                return npcSetting
            }
        }
        return NpcSetting()
    }
    
    /**
     * 保存配置
     */
    static func saveSetting(_ bean: NpcSetting){
        let encoder = JSONEncoder()
        let data = try! encoder.encode(bean)
        UserDefaults.standard.set(data, forKey: settingKey)
    }
    
    /**
     * 获取标记是否是打开状态
     */
    static func isOpened() -> Bool {
        return UserDefaults.standard.bool(forKey: isOpenedKey)
    }
    
    /**
     * 设置标记是否是打开状态
     */
    static func setOpened(_ flag: Bool) {
        UserDefaults.standard.set(flag, forKey:isOpenedKey)
    }
    
    /**
     * 获取是否已经设置完成
     */
    static func isSet() -> Bool {
        return UserDefaults.standard.bool(forKey: isSetKey)
    }
    
    /**
     * 获取是否已经设置完成
     */
    static func saveSet(_ flag: Bool) {
        UserDefaults.standard.set(flag, forKey: isSetKey)
    }
}
