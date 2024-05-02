from concurrent import futures

import grpc 
import fmi3_pb2
import fmi3_pb2_grpc

class BouncingBall:

    def __init__(self):
        self.time = 0
        self.h = 1.0
        self.v = 0
        self.dv = -9.81
        self.e = -0.7 # Coefficient of restitution
        self.v_min = 0.1
        self.dt = 0.01
        self.value_references = {
            0: 'time',
            1: 'h',
            2: 'v',
            3: 'v',
            4: 'dv',
            5: 'dv',
            6: 'e',
            7: 'v_min'
        }

    def doStep(self):
            
            self.v += self.dv * self.dt
            self.h -= self.v * self.dt

            # Check for bounce
            if self.h <= 0:
                self.h = -self.h  # Reflect the height value to positive
                self.v = -self.v * self.e  # Reverse and reduce velocity

    def getValues(self, value_references):
        values = []
        for vr in value_references:
            try:
                values.append(getattr(self, self.value_references[vr]))
            except AttributeError:
                print(f"Parameter with value reference '{vr}' not found.")
                return None
        return values
instances = {}

class Fmi3(fmi3_pb2_grpc.fmi3Servicer):

    def instantiateCoSimulation(self, request, context):
        print("Instantiating co-simulation ...")
        key = 0
        while True:
            if key not in instances:
                instances[key] = BouncingBall()
                break
            key += 1
        return fmi3_pb2.Instance(key=key)
    
    def enterInitializationMode(self, request, context):
        print("Entering initialization mode ...")
        print(f"\t Key: {request.key}")
        print(f"\t toleranceDefined: {request.toleranceDefined}")
        print(f"\t tolerance: {request.tolerance}")
        print(f"\t startTime: {request.startTime}")
        print(f"\t stopTimeDefined: {request.stopTimeDefined}")
        print(f"\t stopTime: {request.stopTime}")
        return fmi3_pb2.Empty()
    
    def exitInitializationMode(self, request, context):
        print("Exiting initialization mode..")
        return fmi3_pb2.Empty()
    
    def freeInstance(self, request, context):
        print(f"Freeing instance with key: {request.key}")
        del instances[request.key]
        return fmi3_pb2.Empty()
    
    def doStep(self, request, context):
        instances[request.key].doStep()
        return fmi3_pb2.Empty()
    
    def getFloat64(self, request, context):
        print(f"getting float64 ...")
        values = instances[request.key].getValues(request.valueReferences)
        return fmi3_pb2.getFloat64Reply(values=values)
    
    def terminate(self, request, context):
        print("Terminating")
        return fmi3_pb2.Empty()


def serve():
    port = "50051"
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    fmi3_pb2_grpc.add_fmi3Servicer_to_server(Fmi3(), server)
    server.add_insecure_port("[::]:" + port)
    server.start()
    print("Server started, listening on " + port)
    server.wait_for_termination()


if __name__ == "__main__":
    serve()
