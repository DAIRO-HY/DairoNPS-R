// 仿java
String.prototype.startsWith = function (str) {
    if (str == null || str == "" || this.length == 0
        || str.length > this.length)
        return false;
    if (this.substr(0, str.length) == str)
        return true;
    else
        return false;
    return true;
};

// 仿java
String.prototype.endsWith = function (str) {
    if (str == null || str == "" || this.length == 0
        || str.length > this.length)
        return false;
    if (this.substring(this.length - str.length) == str)
        return true;
    else
        return false;
    return true;
};

/**
 * 数据流量单位换算
 */
Number.prototype.toDataSize = function (fraction = 2) {
    if (this == null) {
        return "0B"
    }
    const value = this
    if (value >= 1024 * 1024 * 1024 * 1024) {
        return (this / (1024 * 1024 * 1024 * 1024)).toFixed(fraction) + "TB"
    }
    if (value >= 1024 * 1024 * 1024) {
        return (this / (1024 * 1024 * 1024)).toFixed(fraction) + "GB"
    }
    if (value >= 1024 * 1024) {
        return (this / (1024 * 1024)).toFixed(fraction) + "MB"
    }
    if (value >= 1024) {
        return (this / (1024)).toFixed(fraction) + "KB"
    }
    return this.toFixed(fraction) + "B"
}

$(function () {
    if ($(".navbar").length > 0) {
        initTopBar();
    }
});


/**
 * 退出登录
 */
function logout() {
    $.ajaxByData("/admin/login/logout").success(() => {
        window.location.href = "/admin/login.html"
    }).delete()
}

function dateFormat(date, pattern = "yyyy-MM-dd hh:mm:ss") {
    const o = {
        "M+": date.getMonth() + 1, // month
        "d+": date.getDate(), // day
        "h+": date.getHours(), // hour
        "m+": date.getMinutes(), // minute
        "s+": date.getSeconds(), // second
        "q+": Math.floor((date.getMonth() + 3) / 3), // quarter
        "S": date.getMilliseconds()
        // millisecond
    };

    if (/(y+)/.test(pattern)) {
        pattern = pattern.replace(RegExp.$1, (date.getFullYear() + "")
            .substr(4 - RegExp.$1.length));
    }

    for (var k in o) {
        if (new RegExp("(" + k + ")").test(pattern)) {
            pattern = pattern.replace(RegExp.$1, RegExp.$1.length == 1 ? o[k] :
                ("00" + o[k]).substr(("" + o[k]).length));
        }
    }
    return pattern;
}

/**
 * 获取url参数
 * @param key
 * @param def 默认值
 * @returns {string}
 */
function getParam(key, def = "") {

    // 获取当前页面的 URL
    const urlParams = new URLSearchParams(window.location.search);

    // 获取单个参数值
    const value = urlParams.get(key);
    if (value == null) {
        return def
    }
    return value
}

function getCookie(name) {
    // 将 cookie 字符串拆分为数组
    const cookieArray = document.cookie.split('; ');

    // 遍历数组查找指定名称的 cookie
    for (let i = 0; i < cookieArray.length; i++) {
        const cookie = cookieArray[i];
        const [cookieName, cookieValue] = cookie.split('=');

        // 如果找到匹配的 cookie 名称，返回其值
        if (cookieName === name) {
            return decodeURIComponent(cookieValue);
        }
    }

    // 如果没有找到，返回 null
    return null;
}

/**
 * 处理分页提交时的顺序
 * @param d
 */
function handel_page_request(d) {
    return {
        page: d.start / d.length + 1,
        pageSize: d.length,
        keyword: d.search.value,
        sortKey: d.columns[d.order[0].column].data,
        sortType: d.order[0].dir,
        draw: d.draw,
    };
}

/**
 * 保存列表页搜索表单的输入内容,以便页面刷新或从详情页返回时恢复
 * @param $form 表单对象
 */
function saveFormState($form) {
    const data = {}
    $form.serializeArray().forEach(item => {
        data[item.name] = item.value
    })
    sessionStorage.setItem("formState:" + location.pathname, JSON.stringify(data))
}

/**
 * 获取上一次保存的搜索表单内容
 * @returns {null|Object}
 */
function getFormState() {
    const json = sessionStorage.getItem("formState:" + location.pathname)
    if (!json) {
        return null
    }
    try {
        return JSON.parse(json)
    } catch (e) {
        return null
    }
}

/**
 * 按省市区镇村的级联关系,依次加载并恢复地址下拉框的选中状态
 * 第一级下拉框的选项需要页面自行提前加载
 * @param state 保存的表单状态,可以为null
 * @param onDone 恢复完成后的回调(无论是否有保存的数据都会被调用)
 */
function restoreAddressCascade(state, onDone) {
    const $levels = $("select[level]").toArray()
        .sort((a, b) => parseInt($(a).attr("level")) - parseInt($(b).attr("level")))

    function next(index, parentId) {
        if (!state || index >= $levels.length) {
            onDone()
            return
        }
        const $sel = $($levels[index])
        const level = parseInt($sel.attr("level"))
        const value = state[$sel.attr("name")]

        const afterOptionsLoaded = () => {
            if (!value || value === "0") {
                onDone()
                return
            }
            $sel.val(value)
            if (level > 1) {
                $sel.show()
            }
            next(index + 1, value)
        }
        if (level === 1) {//第一级选项由页面自行加载,这里只需要设置选中值
            afterOptionsLoaded()
        } else {
            $sel.loadOption("/common/address_dropdown_by_parent_id/" + parentId, afterOptionsLoaded)
        }
    }

    next(0, 0)
}