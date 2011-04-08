//
//  IronJumpAppDelegate.m
//  IronJump
//
//  Created by Filip Kunc on 8/1/10.
//  For license see LICENSE.TXT
//


#import "IronJumpAppDelegate.h"
#import "EAGLView.h"

@implementation IronJumpAppDelegate

@synthesize window, glView;

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions 
{
    [glView startAnimation];
    return YES;
}

- (void)applicationWillTerminate:(UIApplication *)application 
{
    [glView stopAnimation];
}

- (void)dealloc 
{
    [window release];
    [glView release];
    [super dealloc];
}

- (void)applicationDidBecomeActive:(UIApplication *)application 
{
    [glView startAnimation];
}


- (void)applicationWillResignActive:(UIApplication *)application 
{
    [glView stopAnimation];
}

@end

