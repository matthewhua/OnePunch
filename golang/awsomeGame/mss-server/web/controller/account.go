package controller

import (
	"awsomeGame/mss-server/common"
	"awsomeGame/mss-server/constant"
	"awsomeGame/mss-server/web/model"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
)

var DefaultAccountController = &AccountController{}

type AccountController struct {
}

func (a *AccountController) Register(ctx *gin.Context) {
	/**
	1. 获取请求参数
	2. 根据用户名 查询数据库是否有 有 用户名已存在 没有 注册
	3. 告诉前端 注册成功即可
	*/
	req := &model.RegisterReq{}
	err := ctx.ShouldBind(req)
	if err != nil {
		log.Println("参数格式不合法", err)
		ctx.JSON(http.StatusOK, common.Error(constant.InvalidParam, "参数不合法"))
		return
	}
	//一般web服务 错误格式 自定义
	logic.
}