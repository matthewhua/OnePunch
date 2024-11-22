if __name__ == "__main__":
    data = [1, 2]
    for i in data[:]:
        data.append(i + 1)
        print(i)
    print(data)

    