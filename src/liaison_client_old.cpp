#include <iostream>
#include "zenoh.hxx"
#include <condition_variable>
#include <mutex>
#include "fmi3.pb.h"

using namespace zenoh;

int _main(int argc, char **argv) {

   GetOptions opts;
   opts.set_value("foobar");

   std::mutex m;
   std::condition_variable done_signal;
   bool done = false;

   auto on_reply = [](Reply &&reply) {
   auto result = reply.get();
   if (auto sample = std::get_if<Sample>(&result)) {
      std::cout << "Received ('" << sample->get_keyexpr().as_string_view() << "' : '"
                  << sample->get_payload().as_string_view() << "')\n";
   } else if (auto error = std::get_if<ErrorMessage>(&result)) {
      std::cout << "Received an error :" << error->as_string_view() << "\n";
   }
   };

   auto on_done = [&m, &done, &done_signal]() {
      std::lock_guard lock(m);
      done = true;
      done_signal.notify_all();
   };
   
   Config config;
   // take Session from std::variant
   auto session = expect<Session>(open(std::move(config)));

   session.get("rpc/demo/query","foo=89&bar=12", {on_reply, on_done}, opts);
   std::cout << "Should be working\n";
 
   
   std::unique_lock lock(m);
   done_signal.wait(lock, [&done] { return done; });

   return 0;


}

int main(int argc, char **argv) {
    try {
        _main(argc, argv);
    } catch (ErrorMessage e) {
        std::cout << "Received an error :" << e.as_string_view() << "\n";
    }
}