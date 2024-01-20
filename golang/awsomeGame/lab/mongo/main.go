package main

import (
	"context"
	"fmt"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"log"
	"net/url"
)

type Game struct {
	Title  string
	Genre  string
	Rating float64
}

func main() {
	password := "MimaChaojiChang#666"
	encodedPassword := url.QueryEscape(password)
	uri := "mongodb://ydwl:" + encodedPassword + "@1.117.20.50:27017"
	clientOptions := options.Client().ApplyURI(uri)
	client, err := mongo.Connect(context.Background(), clientOptions)
	if err != nil {
		log.Fatal(err)
	}

	// 检查连接是否成功
	err = client.Ping(context.Background(), nil)
	if err != nil {
		log.Fatal(err)
	}

	// 选择数据库
	db := client.Database("clg-game-30004")

	// 获取集合（表）
	collection := db.Collection("games")

	// 创建文档
	game := Game{
		Title:  "Assassin's Creed Valhalla",
		Genre:  "Action",
		Rating: 4.5,
	}

	// 插入文档
	_, err = collection.InsertOne(context.Background(), game)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Document inserted successfully!")

	// 查询文档
	filter := bson.M{"title": "Assassin's Creed Valhalla"}

	var result Game
	err = collection.FindOne(context.Background(), filter).Decode(&result)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Game: %+v\n", result)

	// 更新文档
	update := bson.M{"$set": bson.M{"rating": 4.8}}

	_, err = collection.UpdateOne(context.Background(), filter, update)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Document updated successfully!")

	// 关闭连接
	err = client.Disconnect(context.Background())
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Disconnected from MongoDB!")
}
