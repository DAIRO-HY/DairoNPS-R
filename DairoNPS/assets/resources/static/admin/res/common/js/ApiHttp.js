let maskShowTimes123 = 0

class ApiHttp {
    constructor(url) {
        // const domain = "https://fly.hy-1.cn"
        const domain = ""
        this.url = domain + url
        this.param = new FormData()

        //默认显示等待加载框
        this.isShowWaiting = true
        this.contentType = false
        this.param.append("_clientFlag","1")
        this.param.append("_version","0")
        this.finishFunc = () => {
            //默认结束后什么也不做
        }
    }

    /**
     * 添加参数
     * @param key 参数名
     * @param value 参数值
     */
    add(key, value) {
        if(value == null) {
            return
        }
        this.param.append(key,value)
        return this
    }

    /**
     * 添加参数
     * @param param 参数数据
     */
    addAll(param) {

        // 遍历键值对
        for (const [key, value] of param) {
            if(value == null) {
                continue
            }
            this.param.append(key,value)
        }
        return this
    }

    /**
     * 设置ContentType
     * @param type
     */
    setContentType(type) {
        this.contentType = type
    }

    /**
     * 设置请求成功回调函数
     * @param block 回调函数
     */
    success(block) {
        this.successFunc = block
        return this
    }

    /**
     * 设置请求失败回调函数(服务器端错误)
     * @param block 回调函数
     */
    fail(block) {
        this.failFunc = block
        return this
    }

    /**
     * 设置请求错误回调函数
     * @param block 回调函数
     */
    error(block) {
        this.errorFunc = block
        return this
    }

    /**
     * 设置请求完成回调函数
     * @param block 回调函数
     */
    finish(block) {
        this.finishFunc = block
        return this
    }

    /**
     * 不显示等待框
     */
    hide() {
        this.isShowWaiting = false
        return this
    }

    /**
     * 发起GET请求
     */
    get() {
        this.request("GET")
    }

    /**
     * 发起POST请求
     */
    post() {
        this.request("POST")
    }

    /**
     * 发起DELETE请求
     */
    delete() {
        this.request("DELETE")
    }

    /**
     * 发起PUT请求
     */
    put() {
        this.request("PUT")
    }

    /**
     * 发起请求
     * @param method 请求方式
     */
    request(method) {
        this.addMask()
        // let urlParam = "_clientFlag=0&_version=0&"
        // for (let key in this.param) {
        //     const value = this.param[key]
        //     if (value == null || value === "") {
        //         continue
        //     }
        //     if (Array.isArray(value)) {//如果这是一个数组
        //         value.forEach(item => {
        //             urlParam += key + "=" + encodeURIComponent(item) + "&"
        //         })
        //     } else {
        //         urlParam += key + "=" + encodeURIComponent(value) + "&"
        //     }
        // }
        // if (urlParam !== "") {//删除最后一个&
        //     urlParam = urlParam.substring(0, urlParam.length - 1)
        // }

        //将数组参数转成单个参数之后使用逗号拼接
        const formDataToSearchParams = function(formData) {
            const map = new Map();
            for (const [key, value] of formData.entries()) {
                if (!map.has(key)) {
                    map.set(key, []);
                }
                map.get(key).push(value);
            }

            const params = new URLSearchParams();

            for (const [key, values] of map) {
                if (values.length === 1) {
                    params.append(key, values[0]);
                } else {
                    params.append(key, values.join(","));
                }
            }
            return params;
        }

        let param
        if(this.contentType === "application/x-www-form-urlencoded"){
            param = formDataToSearchParams(this.param)
        } else {
            param = this.param
        }
        $.ajax({
            url: this.url,
            method: method,
            // data: urlParam,
            data: param,
            dataType: "TEXT",
            processData: false,  // 告诉jQuery不要处理发送的数据
            contentType: this.contentType,  // 告诉jQuery不要设置Content-Type请求头
            success: resText => {
                this.removeMask()
                let data = null
                try {
                    data = JSON.parse(resText)
                } catch {
                    data = resText
                }
                if (this.successFunc) {
                    this.successFunc(data)
                }
            },
            error: xhr => {
                this.removeMask()
                if (xhr.status === 401) {
                    window.location.href = "/admin/login.html"
                    return
                }
                const resText = xhr.responseText
                if (resText === undefined) {
                    alert("网络连接失败")
                    return
                }
                let data = null
                try {
                    data = JSON.parse(resText)
                } catch {
                }
                if (data == null) {//数据解析失败
                    alert(resText)
                    return
                }
                if (data.code === undefined) {//非业务错误
                    alert(resText)
                    return
                }
                if (data.code === 2) {//单项目检查错误
                    const fieldError = data.data
                    this.addFiledError(fieldError)
                    return
                }
                if (this.failFunc) {
                    this.failFunc(data)
                } else {
                    alert(data.msg)
                }
            },
            complete: () => {
                if (this.finishFunc) {
                    this.finishFunc()
                }
            }
        })
    }

    /**
     * 添加验证失败消息
     * @param fieldError
     */
    addFiledError(fieldError) {
        for (let key in fieldError) {
            const name = key.replace(/_([a-zA-Z])/g, (_, c) => c.toUpperCase());
            const messages = fieldError[key]
            const error = messages.join("；") + "。"
            const $input = $(`[name="${name}"]`)
            $input.addClass("is-invalid")
            const $parent = $input.parent()
            $parent.append(`<span class="text-danger" error-valid>${error}</span>`)
        }
        $(".is-invalid").first().focus()
    }

    addMask() {
        if (!this.isShowWaiting) {
            return
        }
        maskShowTimes123++
        if (maskShowTimes123 > 1) {
            return
        }
        const MASK_HTML =
            `<div class="ajax-mask">
            <div class="ajax-mask-animation"></div>
         </div>`
        $("body").append(MASK_HTML)
    }

    removeMask() {
        if (!this.isShowWaiting) {
            return
        }
        maskShowTimes123--
        if (maskShowTimes123 === 0) {
            $(".ajax-mask").remove()
        }
    }
}