def rpc_call(mapping, name):
    def wrapper(func):
        return func
    mapping[name] = wrapper
    return wrapper

class RPC:
    mapping = {}

    @rpc_call(mapping, "get_me")
    def get_me():
        print("get_me")

print(RPC.mapping["get_me"]())

