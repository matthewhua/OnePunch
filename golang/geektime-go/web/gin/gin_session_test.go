package gin

import (
	"github.com/gin-contrib/sessions"
	"github.com/gin-contrib/sessions/cookie"
	"github.com/gin-gonic/gin"

	"testing"
)

func TestGinSession(t *testing.T) {
	r := gin.Default()
	store := cookie.NewStore([]byte("secret"))
	r.Use(sessions.Sessions("mysession", store))

	r.GET("/hello", func(ctx *gin.Context) {
		session := sessions.Default(ctx)

		if session.Get("hello") != "world" {
			session.Set("hello", "world")
			session.Save()
		}

		ctx.JSON(200, gin.H{"hello": session.Get("hello")})
	})
	r.Run(":8080")
}
