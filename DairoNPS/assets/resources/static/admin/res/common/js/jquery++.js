$(function () {

    /**
     * 绑定表单数据扩展函数
     * @param data
     */
    $.fn.bindValue = function (data) {

        //用来保存表单中的name,去除重复
        const nameSet = new Set()
        this.find(":input[name]").each((_, obj) => {
            const $obj = $(obj)
            const name = $obj.attr("name")
            nameSet.add(name)
        })

        nameSet.forEach(name => {
            let value = data[name]
            if (value == null) {
                value = ""
            }
            const $formInputs = this.find(":input[name='" + name + "']")

            //这是复选框
            if ($formInputs[0].tagName === "INPUT" && $formInputs[0].type === "checkbox") {
                if (!Array.isArray(value)) {//如果值不是一个数组
                    value = [value.toString()]
                }
                const valueSet = new Set(value)
                $formInputs.each((_, checkbox) => {
                    checkbox.checked = valueSet.has(checkbox.value)
                })
                return true
            }
            $formInputs.val(value.toString())
        })
    }

    /**
     * 初始化页面数据专用
     * @param success 成功回调
     */
    $.fn.initData = function (success) {
        $.ajaxByData(location.href).success(data => {
            if (success) {
                success(data)
            } else {
                this.bindValue(data)
            }
        }).post()
    }

    $.ajaxByData = function (url) {
        const http = new ApiHttp(url)
        http.setContentType("application/x-www-form-urlencoded")
        return http
    }

    /**
     * Form表单发起ajax请求
     */
    $.fn.ajaxByForm = function (url) {
        this.find("span[error-valid]").remove()
        this.find(".is-invalid").removeClass("is-invalid")
        if (url === undefined) {
            url = this.attr("action")
        }

        //优先使用form标签设置的提交方式
        let contentType =  false
        const enctype = this.attr("enctype")
        if(enctype === undefined){//没有设置enctype情况下
            contentType = "application/x-www-form-urlencoded"
        } else if (enctype.startsWith("multipart/form-data")){//让ajax自己选择提交方式
            contentType = false
        }
        const data = this.formData(this)
        const http = new ApiHttp(url)
        http.setContentType(contentType)
        http.addAll(data)
        return http
    }

    // /**
    //  * 获取表单数据
    //  */
    // $.fn.formData = function () {
    //     const data = {}
    //     $.each(this.find(":input"), (i, obj) => {
    //         // debugger
    //         if (obj.name === "") {
    //             return true
    //         }
    //         if ((obj.type === "radio" || obj.type === "checkbox") && !obj.checked) {
    //             return true
    //         }
    //         if (obj.disabled) {
    //             return true
    //         }
    //         if (data.hasOwnProperty(obj.name)) {//如果该key已经存在
    //             let value = data[obj.name]
    //             if (!Array.isArray(value)) {//把值转换成一个数组
    //                 value = [value]
    //                 data[obj.name] = value
    //             }
    //             value.push(obj.value)
    //         } else {
    //             data[obj.name] = obj.value
    //         }
    //     })
    //     return data
    // }

    /**
     * 获取表单数据
     */
    $.fn.formData = function () {
        const formData = new FormData();
        $.each(this.find(":input"), (i, obj) => {
            if (obj.name === "") {
                return true
            }
            if ((obj.type === "radio" || obj.type === "checkbox") && !obj.checked) {
                return true
            }
            if (obj.disabled) {
                return true
            }
            if (obj.type === "file") {//这是一个文件上传控件
                if(obj.files.length > 0){
                    formData.append(obj.name, obj.files[0])
                }
            } else {
                formData.append(obj.name, obj.value)
            }
        })
        return formData
    }


    /**
     * 加载select数据
     * @param source url
     * @param successFun 成功回调函数
     */
    $.fn.loadOption = function (source, successFun) {
        this.find("option[data]").remove()
        if(source == null){
            return
        }
        if (Array.isArray(source)) {//这是一个数组
            source.forEach(item=>{
                this.append(`<option value="${item.value}" data>${item.label}</option>`)
            })
            if (successFun) {
                successFun()
            }
        } else {
            $.ajaxByData(source).success(data => {
                data.forEach(item => {
                    this.append(`<option value="${item.value}" data>${item.label}</option>`)
                })
                if (successFun) {
                    successFun()
                }
            }).get()
        }
    }


    /**
     * 加载checkbox数据
     * @param source url
     * @param successFun 成功回调函数
     */
    $.fn.loadCheckbox = function (source, successFun) {
        this.children().remove()
        const name = this.attr("name")
        $.ajaxByData(source).success(data => {
            data.forEach((item, index) => {
                this.append(
                    `<div class="form-check form-check-inline">
                         <input class="form-check-input" name="${name}" type="checkbox" id="inlineCheckbox-${index}" value="${item.value}">
                         <label class="form-check-label" for="inlineCheckbox-${index}">${item.label}</label>
                     </div>`)
            })
            if (successFun) {
                successFun()
            }
        }).get()
    }
})