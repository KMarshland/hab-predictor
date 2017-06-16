# Sets up a UNIX domain socket to communicate with the ruby scripts in HABMC.
# Activate by running interface_server.sh

import os
import sys
sys.path.insert(0, '/var/app/tawhiri/tawhiri') # Allows importing api from Tawhiri

import socket
import json
import api

socket_file = "sock"

if os.path.exists(socket_file):
    os.remove(socket_file)

# Uses UNIX domain socket, faster than TCP
server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
server.bind(socket_file)

print "Server initialized, waiting for connections..."

server.listen(1)

try:

    while True:

        conn, addr = server.accept()
        print "Recieved connection"

        while True:

            raw_data = conn.recv(1024)

            if raw_data == '':
                print "Connection terminated"
                raise KeyError

            # Communicates via JSON
            data = json.loads(raw_data)

            is_guidance = data['is_guidance']
            include_metadata = data['include_metadata']
            del data['is_guidance'] # Used locally only - do not send to tawhiri
            del data['include_metadata'] # Used locally only - do not send to tawhiri

            # Run prediction, then return results to HABMC
            raw_response = api.run_prediction(api.parse_request(data))

            if include_metadata:
                response = raw_response
            else:
                if is_guidance:
                    response = [raw_response['prediction'][1]['trajectory'][-1]]
                else:
                    response = raw_response['prediction']

            conn.send(json.dumps(response))

except KeyboardInterrupt:
    print "Shutting down server..."
except KeyError:
    print "Shutting down server..."
except api.PredictionException:
    print "ERROR: outside of time range"

# Clean up
os.remove(socket_file)

