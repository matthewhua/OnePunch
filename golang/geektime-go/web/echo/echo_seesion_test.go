package echo

import (
	"github.com/gorilla/sessions"
	"github.com/labstack/echo-contrib/session"
	"github.com/labstack/echo/v4"
	"net/http"
	"testing"
)

func TestSession(t *testing.T) {

	// Echo instance
	e := echo.New()
	e.Use(session.Middleware(sessions.NewCookieStore([]byte("secret"))))

	e.GET("/", func(c echo.Context) error {
		get, _ := session.Get("session", c)
		get.Options = &sessions.Options{
			Path:     "/",
			MaxAge:   86400 * 7,
			HttpOnly: true,
		}
		get.Values["foo"] = "bar"
		get.Save(c.Request(), c.Response())
		return c.NoContent(http.StatusOK)
	})

}
