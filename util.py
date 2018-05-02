uuid_counter = 0
def gen_uuid():
    global uuid_counter
    uuid_counter += 1
    temp_counter = uuid_counter
    uuid = ''
    for i in range(8): # This should be enough uuids to label all the atoms in the universe or some other really big number.
        uuid = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'[temp_counter % 52] + uuid
        temp_counter //= 52
    return uuid