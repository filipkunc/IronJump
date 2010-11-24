//
//  FPMovablePlatform.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 6/27/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameProtocols.h"

@interface FPMovablePlatform : NSObject <FPGameObject> 
{
	float x, y;
	float moveY;
	int widthSegments;
	int heightSegments;
	BOOL isVisible;
}

- (id)initWithWidthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (BOOL)collisionLeftRight:(id<FPGameProtocol>)game;
- (BOOL)collisionUpDown:(id<FPGameProtocol>)game;

@end
