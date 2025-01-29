local list = {"Hello", "test", "World", "value"}

local map = {
	test = "Hello",
	value = "World"
}

for k, v in map do
	print(k, v)
end

for i, v in list do
	print(i, v)
end
