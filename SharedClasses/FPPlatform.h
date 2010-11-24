//
//  FPPlatform.h
//  IronJump
//
//  Created by Filip Kunc on 5/2/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#if TARGET_OS_IPHONE
#import <Foundation/Foundation.h>
#import <CoreGraphics/CoreGraphics.h>
#else
#import <Cocoa/Cocoa.h>
#endif

#import "FPGameProtocols.h"

@interface FPPlatform : NSObject <FPGameObject> 
{
	float x, y;
	int widthSegments;
	int heightSegments;
	BOOL isVisible;
}

- (id)initWithWidthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;

@end
