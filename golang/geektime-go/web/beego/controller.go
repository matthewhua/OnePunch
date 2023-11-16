package beego

import "github.com/beego/beego/v2/server/web"

type UseController struct {
	web.Controller
}

func (c *UseController) GetUser() {
	c.Ctx.WriteString("Hello, I'm Matthew")
}

func (c *UseController) CreateUser() {
	u := &User{}
	err := c.Ctx.BindJSON(u)
	if err != nil {
		c.Ctx.WriteString(err.Error())
		return
	}
	_ = c.Ctx.JSONResp(u)
}

type User struct {
	Name   string
	Gender string
}
