package sql

import (
	"context"
	"database/sql"
	"fmt"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"testing"
	"time"
)

type sqlTestSuite struct {
	suite.Suite

	// 配置字段
	driver string
	dsn    string

	// 初始化字段
	db *sql.DB
}

func (s *sqlTestSuite) SetupSuite() {
	db, err := sql.Open(s.driver, s.dsn)
	if err != nil {
		s.T().Fatal(err)
	}

	s.db = db
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	_, err = s.db.ExecContext(ctx, `
create table if not exists test_model(
    id INTEGER PRIMARY KEY,
	first_name TEXT NOT NULL,
	age INTEGER ,     
     last_name TEXT NOT NULL)`,
	)
	if err != nil {
		s.T().Fatal(err)
	}
}

func (s *sqlTestSuite) TestCRUD() {
	t := s.T()
	db, err := sql.Open("sqlite3", "file:test.db?cache=shared&mode=memory")
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	// 或者 Exec()
	res, err := db.ExecContext(ctx, "INSERT INTO `test_model`( `id`, `first_name`, `age`, `last_name`) VALUES (?, ?, ?, ?)", 1, "Tom", 18, "d")
	if err != nil {
		t.Fatal(err)
	}

	affected, err := res.RowsAffected()
	if err != nil {
		t.Fatal(err)
	}
	if affected != 1 {
		t.Fatal(err)
	}

	rows, err := db.QueryContext(context.Background(),
		"SELECT `id`, `first_name`,`age`, `last_name` FROM `test_model` LIMIT ?", 1)
	if err != nil {
		t.Fatal()
	}
	for rows.Next() {
		tm := &TestModel{}
		err = rows.Scan(&tm.Id, &tm.FirstName, &tm.Age, &tm.LastName)
		// 常见错误，缺了指针
		// err = rows.Scan(tm.Id, tm.FirstName, tm.Age, tm.LastName)
		if err != nil {
			rows.Close()
			t.Fatal(err)
		}
		assert.Equal(t, "Tom", tm.FirstName)
	}
	rows.Close()

	// 或者 Exec(xxx)
	res, err = db.ExecContext(ctx, "UPDATE `test_model` SET `first_name` = 'changed' WHERE `id` = ?", 1)
	if err != nil {
		t.Fatal(err)
	}
	affected, err = res.RowsAffected()
	if err != nil {
		t.Fatal(err)
	}
	if affected != 1 {
		t.Fatal(err)
	}

	row := db.QueryRowContext(context.Background(), "SELECT `id`, `first_name`,`age`, `last_name` FROM `test_model` LIMIT ?", 1)
	if row.Err() != nil {
		t.Fatal(row.Err())
	}
	tm := &TestModel{}

	err = row.Scan(&tm.Id, &tm.FirstName, &tm.Age, &tm.LastName)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, "changed", tm.FirstName)
}

func (s *sqlTestSuite) TearDownTest() {
	_, err := s.db.Exec("delete from test_model")
	if err != nil {
		s.T().Fatal(err)
	}
}

type TestModel struct {
	Id        int64 `form:"auto_increment,primary_key"`
	FirstName string
	Age       int8
	LastName  *sql.NullString
}

func TestSQLite(t *testing.T) {
	suite.Run(t, &sqlTestSuite{
		driver: "sqlite3",
		dsn:    "file:test.db?cache=shared&mode=memory",
	})
}

func TestTimer(t *testing.T) {
	timer := time.NewTimer(0)
	fmt.Println(timer.Stop())
	<-timer.C
}
